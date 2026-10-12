use serde::{Deserialize, Serialize};

/// Источник утверждения — откуда взята информация.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Порядковый номер главы (обязателен).
    pub chapter: u32,

    /// Индекс сцены внутри главы (0-based).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<usize>,

    /// Имя файла главы.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// Номер строки в главе (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,

    /// Цитата, на которой основано утверждение.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub quote: String,
}

impl Source {
    pub fn new(chapter: u32) -> Self {
        Self {
            chapter,
            scene: None,
            file: None,
            line: None,
            quote: String::new(),
        }
    }

    pub fn with_scene(chapter: u32, scene: usize) -> Self {
        Self {
            chapter,
            scene: Some(scene),
            file: None,
            line: None,
            quote: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_with_scene() {
        let mut s = Source::with_scene(1, 2);
        s.quote = "тест".into();
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"scene\":2"));
        assert!(json.contains("\"chapter\":1"));
    }
}