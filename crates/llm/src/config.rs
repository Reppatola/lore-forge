use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Конфигурация LLM: endpoint + маппинг задач на модели.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    #[serde(default = "default_model")]
    pub default_model: String,

    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    #[serde(default = "default_retries")]
    pub retries: u32,

    #[serde(default)]
    pub tasks: BTreeMap<String, String>,
}

fn default_endpoint() -> String { "http://localhost:11434".into() }
fn default_model()    -> String { "qwen3.5:9b".into() }
fn default_timeout()  -> u64    { 300 }
fn default_retries()  -> u32    { 3 }

impl Default for LlmConfig {
    fn default() -> Self {
        let mut tasks = BTreeMap::new();
        tasks.insert("extract_heavy".into(), "qwen2.5-coder:14b".into());
        tasks.insert("extract_light".into(), "qwen3.5:9b".into());
        tasks.insert("extract_long".into(),  "qwen3:8b-64k".into());
        tasks.insert("classify".into(),      "qwen3:8b".into());
        tasks.insert("code".into(),          "maternion/strand-rust-coder:14b-q4_K_M".into());

        Self {
            endpoint: default_endpoint(),
            default_model: default_model(),
            timeout_secs: default_timeout(),
            retries: default_retries(),
            tasks,
        }
    }
}

impl LlmConfig {
    pub fn load_or_default(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let text = text.trim_start_matches('\u{feff}');
                match toml::from_str::<TomlWrapper>(text) {
                    Ok(w) => w.llm,
                    Err(e) => {
                        tracing::warn!(
                            "не удалось распарсить {}: {e}; использую дефолт",
                            path.display()
                        );
                        Self::default()
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "не удалось прочитать {}: {e}; использую дефолт",
                    path.display()
                );
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let wrapper = TomlWrapper { llm: self.clone() };
        let text = toml::to_string_pretty(&wrapper).map_err(|e| {
            crate::error::LlmError::Ollama(format!("toml serialize: {e}"))
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)?;
        Ok(())
    }

    /// Модель для задачи; fallback на default_model.
    pub fn model_for(&self, task: &str) -> String {
        self.tasks
            .get(task)
            .cloned()
            .unwrap_or_else(|| self.default_model.clone())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TomlWrapper {
    llm: LlmConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_tasks() {
        let cfg = LlmConfig::default();
        assert_eq!(cfg.model_for("extract_light"), "qwen3.5:9b");
        assert_eq!(cfg.model_for("unknown"), "qwen3.5:9b");
    }
}
