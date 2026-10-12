use std::time::Duration;

use crate::config::LlmConfig;
use crate::error::{LlmError, Result};
use crate::types::{GenerateOptions, GenerateRequest, GenerateResponse};

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

    /// Проверка, что Ollama отвечает.
    pub async fn ping(&self) -> Result<Vec<String>> {
        #[derive(serde::Deserialize)]
        struct TagsResponse { models: Vec<TagModel> }
        #[derive(serde::Deserialize)]
        struct TagModel { name: String }

        let url = format!("{}/api/tags", self.endpoint);
        let resp = self.http.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(LlmError::Ollama(format!("GET {} -> {}", url, resp.status())));
        }
        let tags: TagsResponse = resp.json().await?;
        Ok(tags.models.into_iter().map(|m| m.name).collect())
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
