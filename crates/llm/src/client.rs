use std::time::Duration;

use crate::config::LlmConfig;
use crate::error::{LlmError, Result};
use crate::types::{GenerateOptions, GenerateRequest, GenerateResponse};

/// Информация о модели.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub size_bytes: u64,
}

impl ModelInfo {
    /// Размер в человекочитаемом виде: "8.4 GB"
    pub fn size_human(&self) -> String {
        let gb = self.size_bytes as f64 / 1_073_741_824.0;
        if gb >= 1.0 {
            format!("{:.1} GB", gb)
        } else {
            let mb = self.size_bytes as f64 / 1_048_576.0;
            format!("{:.0} MB", mb)
        }
    }
}

/// Клиент Ollama.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    http: reqwest::Client,
    endpoint: String,
    retries: u32,
}

impl OllamaClient {
    pub fn new(cfg: &LlmConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(cfg.timeout_secs))
            .build()?;
        Ok(Self {
            http,
            endpoint: cfg.endpoint.trim_end_matches('/').to_string(),
            retries: cfg.retries.max(1),
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Список моделей с размерами.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        #[derive(serde::Deserialize)]
        struct TagsResponse {
            models: Vec<TagModel>,
        }
        #[derive(serde::Deserialize)]
        struct TagModel {
            name: String,
            #[serde(default)]
            size: u64,
        }

        let url = format!("{}/api/tags", self.endpoint);
        let resp = self.http.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(LlmError::Ollama(format!("GET {} -> {}", url, resp.status())));
        }
        let tags: TagsResponse = resp.json().await?;
        Ok(tags
            .models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name,
                size_bytes: m.size,
            })
            .collect())
    }

    /// Упрощённый список имён моделей (обратная совместимость).
    pub async fn ping(&self) -> Result<Vec<String>> {
        Ok(self
            .list_models()
            .await?
            .into_iter()
            .map(|m| m.name)
            .collect())
    }

    /// Простой запрос: prompt + модель, ответ как строка.
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        system: Option<&str>,
        options: GenerateOptions,
    ) -> Result<GenerateResponse> {
        let req = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: system.map(|s| s.to_string()),
            format: None,
            stream: false,
            options,
        };
        self.generate_raw(req).await
    }

    /// С format=json — модель обязуется вернуть валидный JSON.
    pub async fn generate_json(
        &self,
        model: &str,
        prompt: &str,
        system: Option<&str>,
        options: GenerateOptions,
    ) -> Result<GenerateResponse> {
        let req = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: system.map(|s| s.to_string()),
            format: Some("json".into()),
            stream: false,
            options,
        };
        self.generate_raw(req).await
    }

    async fn generate_raw(&self, req: GenerateRequest) -> Result<GenerateResponse> {
        let url = format!("{}/api/generate", self.endpoint);
        let mut last_err: Option<LlmError> = None;

        for attempt in 1..=self.retries {
            if attempt > 1 {
                let backoff = Duration::from_millis(500 * 2u64.pow(attempt - 2));
                tracing::warn!("попытка {} — ждём {:?}", attempt, backoff);
                tokio::time::sleep(backoff).await;
            }

            match self.http.post(&url).json(&req).send().await {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        last_err = Some(LlmError::Ollama(format!(
                            "HTTP {} — {}",
                            status, body
                        )));
                        continue;
                    }
                    match resp.json::<GenerateResponse>().await {
                        Ok(gen) => return Ok(gen),
                        Err(e) => {
                            last_err = Some(LlmError::Http(e));
                            continue;
                        }
                    }
                }
                Err(e) if e.is_timeout() => {
                    last_err = Some(LlmError::Timeout(req.model.clone()));
                    continue;
                }
                Err(e) => {
                    last_err = Some(LlmError::Http(e));
                    continue;
                }
            }
        }

        Err(LlmError::RetriesExhausted(
            self.retries,
            last_err
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown".into()),
        ))
    }
}