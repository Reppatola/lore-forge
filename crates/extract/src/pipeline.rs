use std::time::Instant;

use loreforge_core::Entity;
use loreforge_ingest::Chunk;
use loreforge_llm::{GenerateOptions, LlmCache, OllamaClient};

use crate::error::{ExtractError, Result};
use crate::parser::parse_llm_response;
use crate::prompt::PromptTemplate;
use crate::types::{ChunkExtraction, LlmResponse};
use crate::validator::validate_and_build;

/// Настройки извлечения.
#[derive(Debug, Clone)]
pub struct ExtractConfig {
    /// Сколько раз пытаться при невалидном JSON.
    pub max_attempts: u32,
    /// Модель для извлечения (задача для роутера).
    pub task: String,
    /// Температура. 0.1–0.3 — для структурированного вывода.
    pub temperature: f32,
    /// Максимум токенов в ответе.
    pub num_predict: i32,
    /// Размер контекста.
    pub num_ctx: i32,
    /// Использовать кэш.
    pub use_cache: bool,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            max_attempts: 2,
            task: "extract_heavy".into(),
            temperature: 0.15,
            num_predict: 4096,
            num_ctx: 8192,
            use_cache: true,
        }
    }
}

/// Итог обработки одного чанка (с маппингом в core::Entity).
#[derive(Debug, Clone)]
pub struct ChunkResult {
    pub extraction: ChunkExtraction,
    pub entities: Vec<Entity>,
    /// unresolved + сущности, которые не прошли валидацию.
    pub unresolved: Vec<String>,
}

/// Обработка одного чанка: промпт → LLM → парсинг → валидация.
pub async fn extract_chunk(
    client: &OllamaClient,
    cache: Option<&LlmCache>,
    template: &PromptTemplate,
    chunk: &Chunk,
    model: &str,
    cfg: &ExtractConfig,
) -> Result<ChunkResult> {
    // 1. Рендер промпта
    let prompt = template.render(&chunk.text, &chunk.prev_context, &chunk.next_context);

    // 2. Проверка кеша
    if cfg.use_cache {
        if let Some(c) = cache {
            if let Some(entry) = c.get(model, &prompt) {
                tracing::debug!("кеш попал: {} / чанк {}", model, chunk.scene_index);
                let parsed = match parse_llm_response(&entry.response) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("кеш содержит невалидный JSON: {e}");
                        LlmResponse { entities: vec![], unresolved: vec![] }
                    }
                };
                let extraction = ChunkExtraction {
                    chapter: chunk.chapter,
                    scene_index: chunk.scene_index,
                    model: model.to_string(),
                    elapsed_secs: 0.0,
                    tokens_out: entry.eval_count,
                    tokens_per_sec: None,
                    raw_response: entry.response.clone(),
                    parsed: parsed.clone(),
                    from_cache: true,
                    attempts: 0,
                };
                return build_result(chunk, extraction, model);
            }
        }
    }

    // 3. Запрос к LLM с retry
    let options = GenerateOptions {
        temperature: cfg.temperature,
        top_p: 0.9,
        num_predict: Some(cfg.num_predict),
        num_ctx: Some(cfg.num_ctx),
    };

    let mut last_err: Option<ExtractError> = None;
    let mut last_raw = String::new();

    for attempt in 1..=cfg.max_attempts {
        let prompt_used = if attempt == 1 {
            prompt.clone()
        } else {
            // strict-версия: усиливаем требования к JSON
            template.render_strict(&chunk.text, &chunk.prev_context, &chunk.next_context)
        };

        let t0 = Instant::now();
        let resp = client
            .generate_json(model, &prompt_used, None, options.clone())
            .await?;
        let elapsed = t0.elapsed().as_secs_f64();

        last_raw = resp.response.clone();

        match parse_llm_response(&resp.response) {
            Ok(parsed) => {
                let extraction = ChunkExtraction {
                    chapter: chunk.chapter,
                    scene_index: chunk.scene_index,
                    model: model.to_string(),
                    elapsed_secs: elapsed,
                    tokens_out: resp.eval_count,
                    tokens_per_sec: resp.eval_tokens_per_sec(),
                    raw_response: resp.response.clone(),
                    parsed: parsed.clone(),
                    from_cache: false,
                    attempts: attempt,
                };

                // запись в кеш
                if cfg.use_cache {
                    if let Some(c) = cache {
                        let entry = loreforge_llm::cache::CacheEntry {
                            model: model.to_string(),
                            prompt: prompt.clone(),
                            response: resp.response.clone(),
                            created_at: chrono::Utc::now().to_rfc3339(),
                            eval_count: resp.eval_count,
                            total_seconds: resp.total_seconds(),
                        };
                        if let Err(e) = c.put(&entry) {
                            tracing::warn!("не удалось записать кеш: {e}");
                        }
                    }
                }

                return build_result(chunk, extraction, model);
            }
            Err(e) => {
                tracing::warn!(
                    "чанк {} попытка {} — невалидный ответ: {e}",
                    chunk.scene_index,
                    attempt
                );
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| ExtractError::InvalidLlmResponse {
        attempts: cfg.max_attempts,
        reason: format!("сырой ответ: {}", &last_raw.chars().take(300).collect::<String>()),
    }))
}

fn build_result(
    chunk: &Chunk,
    extraction: ChunkExtraction,
    model: &str,
) -> Result<ChunkResult> {
    let mut entities = Vec::new();
    let mut unresolved: Vec<String> = extraction
        .parsed
        .unresolved
        .iter()
        .map(|u| format!("{}: {}", u.mention, u.reason))
        .collect();

    for raw in &extraction.parsed.entities {
        match validate_and_build(raw, chunk.chapter, model, 0.9) {
            Ok(e) => entities.push(e),
            Err(e) => {
                let name = raw.names.first().cloned().unwrap_or_else(|| "?".into());
                unresolved.push(format!("{name}: {e}"));
            }
        }
    }

    Ok(ChunkResult {
        extraction,
        entities,
        unresolved,
    })
}

/// Статистика по всей главе.
#[derive(Debug, Default, Clone)]
pub struct ExtractStats {
    pub chunks: usize,
    pub from_cache: usize,
    pub entities_total: usize,
    pub unresolved_total: usize,
    pub elapsed_secs: f64,
}

/// Обработка всех чанков главы последовательно (не грузим Ollama).
pub async fn extract_chapter(
    client: &OllamaClient,
    cache: Option<&LlmCache>,
    template: &PromptTemplate,
    chunks: &[Chunk],
    model: &str,
    cfg: &ExtractConfig,
) -> Result<(Vec<ChunkResult>, ExtractStats)> {
    let t0 = Instant::now();
    let mut results = Vec::with_capacity(chunks.len());
    let mut stats = ExtractStats::default();

    for (i, chunk) in chunks.iter().enumerate() {
        tracing::info!(
            "чанк {}/{} (сцена {}, {} символов)",
            i + 1,
            chunks.len(),
            chunk.scene_index,
            chunk.chars
        );

        let r = extract_chunk(client, cache, template, chunk, model, cfg).await?;

        let cached = if r.extraction.from_cache { " [кеш]" } else { "" };
        tracing::info!(
            "  ✓ {} сущностей, {} unresolved{}, {:.1}s",
            r.entities.len(),
            r.unresolved.len(),
            cached,
            r.extraction.elapsed_secs
        );

        stats.chunks += 1;
        if r.extraction.from_cache {
            stats.from_cache += 1;
        }
        stats.entities_total += r.entities.len();
        stats.unresolved_total += r.unresolved.len();

        results.push(r);
    }

    stats.elapsed_secs = t0.elapsed().as_secs_f64();
    Ok((results, stats))
}