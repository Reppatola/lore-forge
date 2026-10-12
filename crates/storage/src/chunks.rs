use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use loreforge_ingest::Chunk;

use crate::error::Result;
use crate::paths::OutputPaths;

/// Записать чанки главы в JSONL. Один чанк = одна строка.
pub fn write_chunks(
    paths: &OutputPaths,
    chapter: u32,
    chunks: &[Chunk],
) -> Result<()> {
    paths.ensure()?;
    let path = paths.chunk_file(chapter);
    let file = std::fs::File::create(&path)?;
    let mut writer = BufWriter::new(file);
    for chunk in chunks {
        let line = serde_json::to_string(chunk)?;
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    Ok(())
}

/// Прочитать чанки главы из JSONL.
pub fn read_chunks(paths: &OutputPaths, chapter: u32) -> Result<Vec<Chunk>> {
    let path = paths.chunk_file(chapter);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = std::fs::File::open(&path)?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let chunk: Chunk = serde_json::from_str(&line)?;
        out.push(chunk);
    }
    Ok(out)
}

/// Утилита: путь к файлу чанков без создания OutputPaths.
pub fn chunk_file_for(root: &Path, chapter: u32) -> std::path::PathBuf {
    OutputPaths::new(root).chunk_file(chapter)
}
