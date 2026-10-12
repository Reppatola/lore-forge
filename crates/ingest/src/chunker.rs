use serde::{Deserialize, Serialize};

use crate::chapter::{Chapter, Paragraph};

/// Настройки чанкинга.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    /// Сколько абзацев в одной сцене (окне).
    pub window: usize,
    /// Сколько абзацев перекрытия между сценами.
    pub overlap: usize,
    /// Давать ли контекст до/после (±1 абзац).
    pub with_neighbors: bool,
    /// Минимум символов в чанке.
    pub min_chars: usize,
    /// Максимум символов — если абзацев мало, дробим по предложениям.
    pub max_chars: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            window: 4,
            overlap: 1,
            with_neighbors: true,
            min_chars: 200,
            max_chars: 3500,
        }
    }
}

/// Готовый чанк для LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub chapter: u32,
    pub scene_index: usize,
    pub para_start: usize,
    pub para_end: usize,
    pub line_start: u32,
    pub line_end: u32,
    pub text: String,
    pub prev_context: String,
    pub next_context: String,
    pub hash: String,
    pub chars: usize,
}

impl Chunk {
    pub fn full_text(&self) -> String {
        let mut s = String::new();
        if !self.prev_context.is_empty() {
            s.push_str("[КОНТЕКСТ ДО]\n");
            s.push_str(&self.prev_context);
            s.push_str("\n\n[СЦЕНА]\n");
        }
        s.push_str(&self.text);
        if !self.next_context.is_empty() {
            s.push_str("\n\n[КОНТЕКСТ ПОСЛЕ]\n");
            s.push_str(&self.next_context);
        }
        s
    }
}

/// Разбивает текст на абзацы (по пустым строкам).
pub fn split_paragraphs(text: &str) -> Vec<Paragraph> {
    let mut paragraphs = Vec::new();
    let mut current = String::new();
    let mut current_start = 0u32;
    let mut current_end = 0u32;
    let mut line_num = 0u32;

    for line in text.lines() {
        line_num += 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            flush_para(&mut paragraphs, &mut current, current_start, current_end);
            continue;
        }

        if current.trim().is_empty() {
            current_start = line_num;
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(trimmed);
        current_end = line_num;
    }

    flush_para(&mut paragraphs, &mut current, current_start, current_end);
    paragraphs
}

fn flush_para(
    paras: &mut Vec<Paragraph>,
    buf: &mut String,
    start: u32,
    end: u32,
) {
    let trimmed = buf.trim().to_string();
    if trimmed.is_empty() {
        buf.clear();
        return;
    }
    let is_dialogue = trimmed.starts_with('—') || trimmed.starts_with("--");
    paras.push(Paragraph {
        index: paras.len(),
        line_start: start,
        line_end: end,
        text: trimmed,
        is_dialogue,
    });
    buf.clear();
}

/// Главная функция: глава → список чанков.
/// Если абзацев < 3 — используется fallback-режим (по предложениям).
pub fn chunk_chapter(chapter: &Chapter, cfg: &ChunkConfig) -> Vec<Chunk> {
    let paragraphs = split_paragraphs(&chapter.text);

    // Fallback: текст без пустых строк
    if paragraphs.len() < 3 {
        return chunk_fallback(chapter, &paragraphs, cfg);
    }

    chunk_by_paragraphs(chapter, &paragraphs, cfg)
}

/// Обычный режим: окна из абзацев.
fn chunk_by_paragraphs(
    chapter: &Chapter,
    paragraphs: &[Paragraph],
    cfg: &ChunkConfig,
) -> Vec<Chunk> {
    let mut chunks: Vec<Chunk> = Vec::new();
    let step = cfg.window.saturating_sub(cfg.overlap).max(1);
    let mut start = 0usize;
    let mut scene_index = 0usize;

    while start < paragraphs.len() {
        let end = (start + cfg.window).min(paragraphs.len());
        let slice = &paragraphs[start..end];
        let text = slice
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let chars = text.chars().count();

        let prev_context = if cfg.with_neighbors && start > 0 {
            paragraphs[start - 1].text.clone()
        } else {
            String::new()
        };
        let next_context = if cfg.with_neighbors && end < paragraphs.len() {
            paragraphs[end].text.clone()
        } else {
            String::new()
        };

        let hash = compute_chunk_hash(chapter.number, &text);

        chunks.push(Chunk {
            chapter: chapter.number,
            scene_index,
            para_start: start,
            para_end: end - 1,
            line_start: slice.first().unwrap().line_start,
            line_end: slice.last().unwrap().line_end,
            text,
            prev_context,
            next_context,
            hash,
            chars,
        });

        scene_index += 1;
        if end == paragraphs.len() {
            break;
        }
        start += step;
    }

    chunks
}

/// Fallback: режем по предложениям, собираем окна ~max_chars.
fn chunk_fallback(
    chapter: &Chapter,
    paragraphs: &[Paragraph],
    cfg: &ChunkConfig,
) -> Vec<Chunk> {
    // Собираем весь текст (склеиваем абзацы, даже если их 1)
    let full = paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    if full.trim().is_empty() {
        return Vec::new();
    }

    // Если текст короткий — один чанк
    let full_chars = full.chars().count();
    if full_chars <= cfg.max_chars {
        let hash = compute_chunk_hash(chapter.number, &full);
        return vec![Chunk {
            chapter: chapter.number,
            scene_index: 0,
            para_start: 0,
            para_end: paragraphs.len().saturating_sub(1),
            line_start: 1,
            line_end: chapter.line_count,
            text: full.clone(),
            prev_context: String::new(),
            next_context: String::new(),
            hash,
            chars: full_chars,
        }];
    }

    // Режем на предложения
    let sentences = split_sentences(&full);
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut current = String::new();
    let mut idx = 0usize;
    let mut scene_index = 0usize;

    // keep track for context
    let mut prev_sentence: Option<String> = None;

    for (i, sent) in sentences.iter().enumerate() {
        let new_len = current.chars().count() + sent.chars().count();
        if new_len > cfg.max_chars && !current.is_empty() {
            // зафиксировать чанк
            let hash = compute_chunk_hash(chapter.number, &current);
            let next_context = sentences.get(i).cloned().unwrap_or_default();
            chunks.push(Chunk {
                chapter: chapter.number,
                scene_index,
                para_start: 0,
                para_end: 0,
                line_start: 1,
                line_end: chapter.line_count,
                text: current.trim().to_string(),
                prev_context: prev_sentence.clone().unwrap_or_default(),
                next_context,
                hash,
                chars: current.chars().count(),
            });
            prev_sentence = Some(sent.clone());
            current.clear();
            scene_index += 1;
            idx += 1;
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(sent);
    }

    // хвост
    if !current.trim().is_empty() {
        let hash = compute_chunk_hash(chapter.number, &current);
        chunks.push(Chunk {
            chapter: chapter.number,
            scene_index,
            para_start: 0,
            para_end: 0,
            line_start: 1,
            line_end: chapter.line_count,
            text: current.trim().to_string(),
            prev_context: prev_sentence.unwrap_or_default(),
            next_context: String::new(),
            hash,
            chars: current.chars().count(),
        });
    }

    let _ = idx;
    chunks
}

/// Наивная, но быстрая разбивка на предложения.
/// Точка/!/?/… + пробел + заглавная буква (или конец строки).
fn split_sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        buf.push(c);

        let is_terminal = matches!(c, '.' | '!' | '?' | '…' | '—') // тире часто = реплика
            && !buf.trim_end().ends_with("т. е.")
            && !buf.trim_end().ends_with("т. д.");

        if !is_terminal {
            continue;
        }

        // Проверяем, что дальше пробел + заглавная (или конец)
        let next = chars.peek();
        match next {
            None => {
                out.push(buf.trim().to_string());
                buf.clear();
            }
            Some(&nc) if nc.is_whitespace() => {
                // пропускаем пробелы
                let mut k = chars.clone();
                k.next(); // пробел
                while let Some(&wc) = k.peek() {
                    if wc.is_whitespace() {
                        k.next();
                    } else {
                        break;
                    }
                }
                let uc = k.peek().copied();
                let is_upper = uc.map(|u| u.is_uppercase()).unwrap_or(true);
                if is_upper {
                    out.push(buf.trim().to_string());
                    buf.clear();
                }
            }
            _ => {}
        }
    }

    if !buf.trim().is_empty() {
        out.push(buf.trim().to_string());
    }

    out
}

fn compute_chunk_hash(chapter: u32, text: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&chapter.to_le_bytes());
    hasher.update(text.as_bytes());
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn mk_chapter(text: &str) -> Chapter {
        Chapter {
            number: 1,
            title: "test".into(),
            path: PathBuf::from("test.md"),
            text: text.into(),
            hash: "0".into(),
            line_count: text.lines().count() as u32,
        }
    }

    #[test]
    fn split_simple() {
        let text = "Абзац 1.\n\nАбзац 2.\n\nАбзац 3.";
        let paras = split_paragraphs(text);
        assert_eq!(paras.len(), 3);
    }

    #[test]
    fn dialogue_kept() {
        let text = "Князь вошёл.\n\n— Привет, — сказал он.\n\nОна улыбнулась.";
        let paras = split_paragraphs(text);
        assert_eq!(paras.len(), 3);
        assert!(paras[1].is_dialogue);
    }

    #[test]
    fn chunking_windows() {
        let text = (1..=10)
            .map(|i| format!("Абзац номер {}.\n", i))
            .collect::<Vec<_>>()
            .join("\n");
        let ch = mk_chapter(&text);
        let cfg = ChunkConfig {
            window: 3,
            overlap: 1,
            with_neighbors: true,
            min_chars: 0,
            max_chars: 100000,
        };
        let chunks = chunk_chapter(&ch, &cfg);
        assert!(chunks.len() >= 3);
    }

    #[test]
    fn fallback_splits_long_text() {
        // один абзац, длинный
        let text = (1..=200)
            .map(|i| format!("Это предложение номер {}. ", i))
            .collect::<String>();
        let ch = mk_chapter(&text);
        let cfg = ChunkConfig {
            max_chars: 500,
            ..Default::default()
        };
        let chunks = chunk_chapter(&ch, &cfg);
        assert!(chunks.len() > 3, "ожидалось >3 чанков, получено {}", chunks.len());
    }

    #[test]
    fn split_sentences_basic() {
        let s = split_sentences("Первое. Второе! Третье?");
        assert_eq!(s.len(), 3);
    }
}
