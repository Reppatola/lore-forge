use thiserror::Error;

pub type Result<T> = std::result::Result<T, ExtractError>;

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("llm error: {0}")]
    Llm(#[from] loreforge_llm::LlmError),

    #[error("ingest error: {0}")]
    Ingest(#[from] loreforge_ingest::IngestError),

    #[error("core error: {0}")]
    Core(#[from] loreforge_core::CoreError),

    #[error("невалидный ответ LLM после {attempts} попыток: {reason}")]
    InvalidLlmResponse { attempts: u32, reason: String },

    #[error("промпт-шаблон не найден: {0}")]
    PromptNotFound(String),
}
