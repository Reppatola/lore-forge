use thiserror::Error;

pub type Result<T> = std::result::Result<T, IngestError>;

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid chapter filename: {0}")]
    InvalidFilename(String),

    #[error("no input files found in {0}")]
    NoInput(String),

    #[error("core error: {0}")]
    Core(#[from] loreforge_core::CoreError),
}
