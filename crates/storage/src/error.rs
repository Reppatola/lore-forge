use thiserror::Error;

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("ingest error: {0}")]
    Ingest(#[from] loreforge_ingest::IngestError),
}
