use thiserror::Error;

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid id: {0}")]
    InvalidId(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("unknown entity kind: {0}")]
    UnknownKind(String),
}