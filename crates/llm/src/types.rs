use serde::{Deserialize, Serialize};

/// Запрос к /api/generate.
#[derive(Debug, Clone, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub stream: bool,
    pub options: GenerateOptions,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateOptions {
    pub temperature: f32,
    pub top_p: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<i32>,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            temperature: 0.2,
            top_p: 0.9,
            num_predict: None,
            num_ctx: Some(8192),
        }
    }
}

/// Ответ от /api/generate (stream=false).
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub response: String,
    #[serde(default)]
    pub done: bool,

    #[serde(default, rename = "total_duration")]
    pub total_duration_ns: Option<u64>,
    #[serde(default, rename = "load_duration")]
    pub load_duration_ns: Option<u64>,
    #[serde(default, rename = "prompt_eval_count")]
    pub prompt_eval_count: Option<u32>,
    #[serde(default, rename = "eval_count")]
    pub eval_count: Option<u32>,
    #[serde(default, rename = "eval_duration")]
    pub eval_duration_ns: Option<u64>,
}

impl GenerateResponse {
    pub fn eval_tokens_per_sec(&self) -> Option<f64> {
        let count = self.eval_count? as f64;
        let dur = self.eval_duration_ns? as f64 / 1_000_000_000.0;
        if dur > 0.0 { Some(count / dur) } else { None }
    }

    pub fn total_seconds(&self) -> Option<f64> {
        Some(self.total_duration_ns? as f64 / 1_000_000_000.0)
    }
}
