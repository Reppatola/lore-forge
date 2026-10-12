use thiserror::Error;

pub type Result<T> = std::result::Result<T, LlmError>;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("ollama вернула ошибку: {0}")]
    Ollama(String),

    #[error("модель не отвечает: {0}")]
    Timeout(String),

    #[error("невалидный JSON в ответе модели: {0}\n---- RAW ----\n{1}")]
    InvalidJson(String, String),

    #[error("модель {0} не найдена в конфиге задач")]
    UnknownTask(String),

    #[error("все попытки исчерпаны ({0}): {1}")]
    RetriesExhausted(u32, String),
}
