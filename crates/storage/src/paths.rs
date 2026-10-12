use std::path::{Path, PathBuf};

/// Все стандартные пути проекта.
#[derive(Debug, Clone)]
pub struct OutputPaths {
    pub root: PathBuf,
    pub input: PathBuf,
    pub output: PathBuf,
    pub chunks: PathBuf,
    pub meta: PathBuf,
    pub manifest: PathBuf,
}

impl OutputPaths {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let output = root.join("data").join("output");
        let chunks = output.join("chunks");
        let meta = output.join("_meta");
        let manifest = meta.join("manifest.json");
        Self {
            root: root.clone(),
            input: root.join("data").join("input"),
            output,
            chunks,
            meta,
            manifest,
        }
    }

    pub fn ensure(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.chunks)?;
        std::fs::create_dir_all(&self.meta)?;
        Ok(())
    }

    pub fn chunk_file(&self, chapter: u32) -> PathBuf {
        self.chunks.join(format!("{:03}.jsonl", chapter))
    }
}
