use std::collections::BTreeSet;
use std::path::Path;

use loreforge_extract::types::ChunkExtraction;
use walkdir::WalkDir;

use crate::error::Result;

/// Собирает все unresolved из extracted/*.jsonl и фильтрует те,
/// чьи `mention` уже попали в names сущностей.
pub fn collect_unresolved(
    extracted_dir: &Path,
    known_names: &BTreeSet<String>,
) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    if !extracted_dir.exists() {
        return Ok(out);
    }

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
            for u in &extraction.parsed.unresolved {
                let mention = u.mention.trim();
                if mention.is_empty() {
                    continue;
                }
                // фильтр: если mention совпадает с известным именем — пропускаем
                if known_names.contains(mention) {
                    continue;
                }
                let key = format!("{}:{}", mention, u.reason);
                if seen.contains(&key) {
                    continue;
                }
                seen.insert(key);
                out.push(format!("{}: {}", mention, u.reason));
            }
        }
    }

    Ok(out)
}