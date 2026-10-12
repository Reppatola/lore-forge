use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use loreforge_extract::{extract_chapter, ExtractConfig, PromptTemplate};
use loreforge_ingest::{chunk_chapter, scan_input, ChunkConfig, Manifest};
use loreforge_llm::{LlmCache, LlmConfig, OllamaClient};
use loreforge_merge::merge_all;
use loreforge_storage::{read_chunks, write_chunks, OutputPaths};

/// Встроенный дефолтный промпт (если рядом нет prompts/extract_entities.md).
const DEFAULT_PROMPT: &str = include_str!("../../../prompts/extract_entities.md");

/// Встроенный дефолтный конфиг (если рядом нет loreforge.toml).
const DEFAULT_CONFIG: &str = include_str!("../../../loreforge.toml");

#[derive(Parser)]
#[command(name = "loreforge", version, about = "Сборка JSON-базы из текста истории")]
struct Cli {
    /// Корень проекта. По умолчанию — папка рядом с .exe.
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Создать структуру проекта в текущей папке.
    Init,

    /// Просканировать главы, порезать на чанки, сохранить.
    Ingest {
        #[arg(default_value = "data/input")]
        input: PathBuf,
        #[arg(long)]
        force: bool,
    },

    /// Извлечь сущности через LLM.
    Extract {
        #[arg(long)]
        chapter: Option<u32>,
        #[arg(long)]
        force: bool,
        #[arg(long, default_value = "extract_heavy")]
        task: String,
    },

    /// Слить extracted в единые объекты базы.
    Merge {
        #[arg(long)]
        force: bool,
    },

    /// Полный цикл: ingest + extract + merge.
    Run {
        #[arg(long, default_value = "data/input")]
        input: PathBuf,
        #[arg(long)]
        force: bool,
        #[arg(long, default_value = "extract_heavy")]
        task: String,
    },

    /// Показать статистику базы.
    Stats,

    /// Проверить Ollama и модели.
    Ping,
}

/// Возвращает корень: явно указанный, или папка рядом с .exe,
/// или текущая папка если путь .exe неизвестен.
fn resolve_root(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(p) = explicit {
        return p;
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ошибка: {e:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    let root = resolve_root(cli.root);
    let paths = OutputPaths::new(&root);

    match cli.command {
        Command::Init => cmd_init(&paths),

        Command::Ingest { input, force } => cmd_ingest(&paths, &input, force),

        Command::Extract {
            chapter,
            force,
            task,
        } => cmd_extract(&paths, chapter, force, task).await,

        Command::Merge { force } => cmd_merge(&paths, force),

        Command::Run {
            input,
            force,
            task,
        } => cmd_run(&paths, &input, force, task).await,

        Command::Stats => cmd_stats(&paths),

        Command::Ping => cmd_ping(&paths).await,
    }
}

// ────────────────────────────────────────────────────────────────────
// init
// ────────────────────────────────────────────────────────────────────

fn cmd_init(paths: &OutputPaths) -> Result<()> {
    println!("Инициализация lore-forge в {}", paths.root.display());
    println!();

    let dirs = [
        paths.root.join("data").join("input"),
        paths.root.join("data").join("output"),
        paths.root.join("data").join("cache"),
        paths.root.join("prompts"),
    ];

    for d in &dirs {
        if !d.exists() {
            std::fs::create_dir_all(d)?;
            println!("  + создано: {}", d.display());
        } else {
            println!("  - уже есть: {}", d.display());
        }
    }

    // Дефолтный промпт
    let prompt_path = paths.root.join("prompts").join("extract_entities.md");
    if !prompt_path.exists() {
        std::fs::write(&prompt_path, DEFAULT_PROMPT)?;
        println!("  + создан: {}", prompt_path.display());
    } else {
        println!("  - уже есть: {}", prompt_path.display());
    }

    // Дефолтный конфиг
    let cfg_path = paths.root.join("loreforge.toml");
    if !cfg_path.exists() {
        std::fs::write(&cfg_path, DEFAULT_CONFIG)?;
        println!("  + создан: {}", cfg_path.display());
    } else {
        println!("  - уже есть: {}", cfg_path.display());
    }

    println!();
    println!("Готово! Положи главы в: {}/data/input/", paths.root.display());
    println!("Имена файлов: 001_название.md, 002_название.md, ...");
    println!();
    println!("Дальше:");
    println!("  loreforge ingest    -- порезать на чанки");
    println!("  loreforge extract   -- извлечь через LLM");
    println!("  loreforge merge     -- собрать базу");
    println!("  или сразу: loreforge run");
    Ok(())
}

// ────────────────────────────────────────────────────────────────────
// ingest
// ────────────────────────────────────────────────────────────────────

fn cmd_ingest(paths: &OutputPaths, input: &PathBuf, force: bool) -> Result<()> {
    let input_dir = if input.is_absolute() {
        input.clone()
    } else {
        paths.root.join(input)
    };

    tracing::info!("сканирую {}", input_dir.display());
    let chapters = scan_input(&input_dir)?;

    if chapters.is_empty() {
        anyhow::bail!("не найдено глав в {}", input_dir.display());
    }

    let mut manifest = Manifest::load(&paths.manifest).unwrap_or_default();
    let to_process: Vec<_> = if force {
        chapters.iter().collect()
    } else {
        manifest.changed(&chapters)
    };

    if to_process.is_empty() {
        tracing::info!("все главы актуальны (--force чтобы пересчитать)");
        return Ok(());
    }

    let cfg = ChunkConfig::default();
    let mut total_chunks = 0usize;

    for chapter in to_process {
        let chunks = chunk_chapter(chapter, &cfg);
        write_chunks(paths, chapter.number, &chunks)?;
        manifest.record(chapter, chunks.len(), 0, 0);
        total_chunks += chunks.len();
        tracing::info!(
            "глава {:03} «{}» -> {} чанков",
            chapter.number,
            chapter.title,
            chunks.len()
        );
    }

    manifest.save(&paths.manifest)?;
    tracing::info!("готово. Всего чанков: {}", total_chunks);
    Ok(())
}

// ────────────────────────────────────────────────────────────────────
// extract
// ────────────────────────────────────────────────────────────────────

async fn cmd_extract(
    paths: &OutputPaths,
    chapter_filter: Option<u32>,
    force: bool,
    task: String,
) -> Result<()> {
    let cfg_path = paths.root.join("loreforge.toml");
    let llm_cfg = LlmConfig::load_or_default(&cfg_path);
    let model = llm_cfg.model_for(&task);
    tracing::info!("модель для задачи {}: {}", task, model);

    let client = OllamaClient::new(&llm_cfg)?;
    let cache_dir = paths.root.join("data/cache/llm");
    let cache = LlmCache::new(cache_dir);

    let prompt_path = paths.root.join("prompts/extract_entities.md");
    let template = PromptTemplate::load_or_builtin(&prompt_path, Some(DEFAULT_PROMPT))?;
    tracing::info!("промпт: {}", template.path().display());

    let manifest = Manifest::load(&paths.manifest).unwrap_or_default();
    if manifest.chapters.is_empty() {
        anyhow::bail!("база пуста -- сначала `loreforge ingest`");
    }

    let chapters_to_process: Vec<u32> = match chapter_filter {
        Some(n) => {
            if !manifest.chapters.contains_key(&n) {
                anyhow::bail!("глава {} не найдена в manifest", n);
            }
            vec![n]
        }
        None => manifest.chapters.keys().copied().collect(),
    };

    let extract_cfg = ExtractConfig {
        task: task.clone(),
        use_cache: !force,
        ..Default::default()
    };

    let output_dir = paths.output.join("extracted");
    std::fs::create_dir_all(&output_dir)?;

    let mut total_entities = 0usize;
    let mut total_unresolved = 0usize;
    let mut total_chunks = 0usize;
    let mut total_cached = 0usize;

    for ch_num in &chapters_to_process {
        let chunks = read_chunks(paths, *ch_num)?;
        if chunks.is_empty() {
            tracing::warn!("глава {:03}: нет чанков, пропускаю", ch_num);
            continue;
        }

        tracing::info!("=== глава {:03}: {} чанков ===", ch_num, chunks.len());

        let (results, stats) = extract_chapter(
            &client,
            Some(&cache),
            &template,
            &chunks,
            &model,
            &extract_cfg,
        )
        .await
        .with_context(|| format!("extract_chapter {}", ch_num))?;

        let out_file = output_dir.join(format!("{:03}.jsonl", ch_num));
        let mut lines = String::new();
        for r in &results {
            let json = serde_json::to_string(&r.extraction)?;
            lines.push_str(&json);
            lines.push('\n');
        }
        std::fs::write(&out_file, lines)?;

        // сохранение чистых raw-ответов для отладки
        let raw_dir = paths.output.join("_meta").join("raw");
        std::fs::create_dir_all(&raw_dir)?;
        for r in &results {
            let raw_file = raw_dir.join(format!(
                "{:03}_scene_{:02}.txt",
                ch_num, r.extraction.scene_index
            ));
            std::fs::write(&raw_file, &r.extraction.raw_response)?;
        }

        tracing::info!("записано: {}", out_file.display());

        total_entities += stats.entities_total;
        total_unresolved += stats.unresolved_total;
        total_chunks += stats.chunks;
        total_cached += stats.from_cache;

        tracing::info!(
            "глава {:03}: {} сущностей, {} unresolved, {:.1}s (из кеша: {})",
            ch_num,
            stats.entities_total,
            stats.unresolved_total,
            stats.elapsed_secs,
            stats.from_cache
        );
    }

    tracing::info!("──────────────────────────────");
    tracing::info!("ИТОГО:");
    tracing::info!("  чанков:        {}", total_chunks);
    tracing::info!("  из кеша:       {}", total_cached);
    tracing::info!("  сущностей:     {}", total_entities);
    tracing::info!("  unresolved:    {}", total_unresolved);
    Ok(())
}

// ────────────────────────────────────────────────────────────────────
// merge
// ────────────────────────────────────────────────────────────────────

fn cmd_merge(paths: &OutputPaths, force: bool) -> Result<()> {
    tracing::info!("собираю extracted-записи...");
    let stats = merge_all(paths, force)?;

    tracing::info!("──────────────────────────────");
    tracing::info!("MERGE готово за {:.2}s", stats.elapsed_secs);
    tracing::info!("  глав:              {}", stats.chapters);
    tracing::info!("  записей на входе:  {}", stats.records_in);
    tracing::info!("  уникальных:        {}", stats.entities_unique);
    tracing::info!("  unresolved:        {}", stats.unresolved_count);
    tracing::info!("  база:              {}/entities/", paths.output.display());
    Ok(())
}

// ────────────────────────────────────────────────────────────────────
// run (полный цикл)
// ────────────────────────────────────────────────────────────────────

async fn cmd_run(
    paths: &OutputPaths,
    input: &PathBuf,
    force: bool,
    task: String,
) -> Result<()> {
    tracing::info!("=== RUN: полный цикл ===");
    tracing::info!("");

    tracing::info!("[1/3] ingest");
    cmd_ingest(paths, input, force)?;

    tracing::info!("");
    tracing::info!("[2/3] extract");
    cmd_extract(paths, None, force, task).await?;

    tracing::info!("");
    tracing::info!("[3/3] merge");
    cmd_merge(paths, force)?;

    tracing::info!("");
    tracing::info!("=== RUN завершён ===");
    tracing::info!("База: {}/entities/", paths.output.display());
    Ok(())
}

// ────────────────────────────────────────────────────────────────────
// stats
// ────────────────────────────────────────────────────────────────────

fn cmd_stats(paths: &OutputPaths) -> Result<()> {
    let manifest = Manifest::load(&paths.manifest).unwrap_or_default();
    if manifest.chapters.is_empty() {
        println!("база пуста -- запусти `loreforge ingest`");
        return Ok(());
    }

    println!("--- Главы ---");
    let mut total_chunks = 0;
    for (num, rec) in &manifest.chapters {
        let chunks = read_chunks(paths, *num).unwrap_or_default();
        total_chunks += chunks.len();
        println!(
            "  {:03} «{}» -- {} чанков, ревизия {}",
            num,
            rec.title,
            chunks.len(),
            rec.revision
        );
    }
    println!(
        "  Глав: {}, Чанков: {}",
        manifest.chapters.len(),
        total_chunks
    );

    let index_path = paths.output.join("_meta").join("index.json");
    if index_path.exists() {
        let text = std::fs::read_to_string(&index_path)?;
        let idx: serde_json::Value = serde_json::from_str(&text)?;
        println!();
        println!("--- База (merge) ---");
        if let Some(total) = idx.get("total_entities") {
            println!("  Всего сущностей: {}", total);
        }
        if let Some(unres) = idx.get("unresolved_count") {
            println!("  Unresolved:      {}", unres);
        }
        if let Some(by_type) = idx.get("by_type").and_then(|v| v.as_object()) {
            println!("  По типам:");
            for (k, v) in by_type {
                let count = v.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
                println!("    {:>12}: {}", k, count);
            }
        }
    } else {
        println!();
        println!("merge ещё не запускался: `loreforge merge`");
    }

    Ok(())
}

// ────────────────────────────────────────────────────────────────────
// ping
// ────────────────────────────────────────────────────────────────────

async fn cmd_ping(paths: &OutputPaths) -> Result<()> {
    let cfg_path = paths.root.join("loreforge.toml");
    let cfg = LlmConfig::load_or_default(&cfg_path);
    println!("endpoint: {}", cfg.endpoint);
    println!("default:  {}", cfg.default_model);

    let client = OllamaClient::new(&cfg)?;
    let models = client.ping().await?;
    println!("[ok] Ollama жива, доступно моделей: {}", models.len());
    for m in &models {
        println!("    * {}", m);
    }

    let router = loreforge_llm::ModelRouter::new(cfg.clone());
    println!();
    println!("-- Задачи --");
    for (task, model) in router.tasks() {
        println!("  {:>14} -> {}", task, model);
    }
    Ok(())
}