use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Одна глава истории: файл на диске + метаданные.
#[derive(Debug, Clone)]
pub struct Chapter {
    /// Порядковый номер, извлечённый из имени файла (001, 002, ...).
    pub number: u32,
    /// Заголовок (остаток имени файла после номера, без расширения).
    pub title: String,
    /// Полный путь к файлу.
    pub path: PathBuf,
    /// Сырой текст.
    pub text: String,
    /// Хеш содержимого (blake3, hex).
    pub hash: String,
    /// Количество строк.
    pub line_count: u32,
}

/// Абзац — единица смысла.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    /// Индекс абзаца в главе (0-based).
    pub index: usize,
    /// Первая строка (1-based, включительно).
    pub line_start: u32,
    /// Последняя строка (1-based, включительно).
    pub line_end: u32,
    /// Текст абзаца (склеенные строки через \n).
    pub text: String,
    /// Это диалог? (начинается с — или --)
    pub is_dialogue: bool,
}

impl Paragraph {
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}
