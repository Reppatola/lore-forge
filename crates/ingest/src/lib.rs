//! Ingestion: чтение глав, парсинг, чанкинг, manifest.

pub mod chapter;
pub mod chunker;
pub mod error;
pub mod manifest;
pub mod reader;

pub use chapter::{Chapter, Paragraph};
pub use chunker::{chunk_chapter, split_paragraphs, Chunk, ChunkConfig};
pub use error::{IngestError, Result};
pub use manifest::{ChapterRecord, Manifest};
pub use reader::{load_chapter, scan_input};

/// Имя файла манифеста (внутри data/output/_meta/).
pub const MANIFEST_FILE: &str = "manifest.json";

/// Загрузить манифест по пути.
pub fn load_manifest(path: &std::path::Path) -> Result<Manifest> {
    Manifest::load(path)
}

/// Сохранить манифест по пути.
pub fn save_manifest(path: &std::path::Path, m: &Manifest) -> Result<()> {
    m.save(path)
}
