use thiserror::Error;

pub type Result<T> = std::result::Result<T, MergeError>;

#[derive(Debug, Error)]
pub enum MergeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("extract error: {0}")]
    Extract(#[from] loreforge_extract::ExtractError),

    #[error("core error: {0}")]
    Core(#[from] loreforge_core::CoreError),

    #[error("нет папки extracted: {0}")]
    NoExtracted(String),

    #[error("не найдено ни одного файла extracted в {0}")]
    EmptyExtracted(String),
}
