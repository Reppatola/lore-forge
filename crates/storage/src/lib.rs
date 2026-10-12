//! Storage: сохранение чанков и манифеста в JSONL/JSON.

pub mod chunks;
pub mod error;
pub mod paths;

pub use chunks::{write_chunks, read_chunks};
pub use error::{StorageError, Result};
pub use paths::OutputPaths;
