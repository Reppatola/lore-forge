use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Кеш ответов LLM. Ключ = blake3(модель + промпт + текст).
/// Хранится в `<root>/<first2>/<hash>.json`.
#[derive(Debug, Clone)]
pub struct LlmCache {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub model: String,
    pub prompt: String,
    pub response: String,
    pub created_at: String,
    pub eval_count: Option<u32>,
    pub total_seconds: Option<f64>,
}

impl LlmCache {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn key(&self, model: &str, prompt: &str) -> String {
        let mut h = blake3::Hasher::new();
        h.update(model.as_bytes());
        h.update(b"\x00");
        h.update(prompt.as_bytes());
        h.finalize().to_hex().to_string()
    }

    fn path_for(&self, hash: &str) -> PathBuf {
        let prefix = &hash[..2];
        self.root.join(prefix).join(format!("{hash}.json"))
    }

    pub fn get(&self, model: &str, prompt: &str) -> Option<CacheEntry> {
        let key = self.key(model, prompt);
        let path = self.path_for(&key);
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str::<CacheEntry>(&text).ok()
    }

    pub fn put(&self, entry: &CacheEntry) -> Result<()> {
        let key = self.key(&entry.model, &entry.prompt);
        let path = self.path_for(&key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(entry)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = LlmCache::new(tmp.path());
        let e = CacheEntry {
            model: "m".into(),
            prompt: "p".into(),
            response: "r".into(),
            created_at: "now".into(),
            eval_count: Some(10),
            total_seconds: Some(1.0),
        };
        cache.put(&e).unwrap();
        let got = cache.get("m", "p").unwrap();
        assert_eq!(got.response, "r");
    }
}
