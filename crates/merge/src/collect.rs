use std::path::Path;

use loreforge_extract::types::ChunkExtraction;
use loreforge_extract::validator::validate_and_build;
use loreforge_core::Entity;
use walkdir::WalkDir;

use crate::error::{MergeError, Result};

/// дна запись о сущности + метаданные, откуда она пришла.
#[derive(Debug, Clone)]
pub struct CollectedEntity {
    pub chapter: u32,
    pub scene_index: usize,
    pub model: String,
    pub entity: Entity,
}

/// итает все `*.jsonl` из папки extracted и собирает все сущности.
pub fn collect_all(extracted_dir: &Path) -> Result<Vec<CollectedEntity>> {
    if !extracted_dir.exists() {
        return Err(MergeError::NoExtracted(extracted_dir.display().to_string()));
    }

    let mut out = Vec::new();

    for entry in WalkDir::new(extracted_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }

        let text = std::fs::read_to_string(path)?;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let extraction: ChunkExtraction = serde_json::from_str(line)?;

            for raw in &extraction.parsed.entities {
                match validate_and_build(
                    raw,
                    extraction.chapter,
                    &extraction.model,
                    0.9,
                ) {
                    Ok(mut entity) => {
                        // описываем scene в sources
                        for src in &mut entity.sources {
                            src.scene = Some(extraction.scene_index);
                        }
                        out.push(CollectedEntity {
                            chapter: extraction.chapter,
                            scene_index: extraction.scene_index,
                            model: extraction.model.clone(),
                            entity,
                        });
                    }
                    Err(e) => {
                        tracing::warn!(
                            "глава {}, сцена {}: не удалось собрать сущность: {e}",
                            extraction.chapter,
                            extraction.scene_index
                        );
                    }
                }
            }
        }
    }

    if out.is_empty() {
        return Err(MergeError::EmptyExtracted(
            extracted_dir.display().to_string(),
        ));
    }

    Ok(out)
}
