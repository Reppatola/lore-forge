use std::collections::BTreeMap;
use std::path::Path;

use loreforge_core::{Entity, EntityKind};
use loreforge_storage::OutputPaths;

use crate::error::Result;

/// Записывает один объект в `entities/<folder>/<id>.json`.
pub fn write_entity(paths: &OutputPaths, entity: &Entity) -> Result<()> {
    let folder = paths.output.join("entities").join(entity.kind.folder());
    std::fs::create_dir_all(&folder)?;
    let file = folder.join(format!("{}.json", entity.id.as_str()));
    let text = serde_json::to_string_pretty(entity)?;
    std::fs::write(file, text)?;
    Ok(())
}

/// Записывает все объекты как jsonl-дамп для отладки.
pub fn write_jsonl_dump(paths: &OutputPaths, entities: &[Entity]) -> Result<()> {
    let dump = paths.output.join("entities.jsonl");
    let mut lines = String::new();
    for e in entities {
        lines.push_str(&serde_json::to_string(e)?);
        lines.push('\n');
    }
    std::fs::write(dump, lines)?;
    Ok(())
}

/// Собирает `index.json` — оглавление всей базы.
pub fn write_index(
    paths: &OutputPaths,
    entities: &[Entity],
    chapters: &[u32],
    unresolved_count: usize,
) -> Result<()> {
    let meta_dir = paths.output.join("_meta");
    std::fs::create_dir_all(&meta_dir)?;

    #[derive(serde::Serialize)]
    struct TypeStat {
        count: usize,
        ids: Vec<String>,
    }

    let mut by_type: BTreeMap<String, TypeStat> = BTreeMap::new();
    for e in entities {
        let key = format!("{:?}", e.kind).to_lowercase();
        let entry = by_type.entry(key).or_insert(TypeStat {
            count: 0,
            ids: Vec::new(),
        });
        entry.count += 1;
        entry.ids.push(e.id.as_str().to_string());
    }

    #[derive(serde::Serialize)]
    struct Index {
        version: String,
        generated_at: String,
        chapters: Vec<u32>,
        total_entities: usize,
        unresolved_count: usize,
        by_type: BTreeMap<String, TypeStat>,
    }

    let index = Index {
        version: env!("CARGO_PKG_VERSION").to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        chapters: chapters.to_vec(),
        total_entities: entities.len(),
        unresolved_count,
        by_type,
    };

    let text = serde_json::to_string_pretty(&index)?;
    std::fs::write(meta_dir.join("index.json"), text)?;
    Ok(())
}

/// Записывает unresolved-упоминания в `_meta/unresolved.jsonl`.
pub fn write_unresolved(paths: &OutputPaths, lines: &[String]) -> Result<()> {
    let meta_dir = paths.output.join("_meta");
    std::fs::create_dir_all(&meta_dir)?;
    let content = lines.join("\n");
    std::fs::write(meta_dir.join("unresolved.jsonl"), content)?;
    Ok(())
}

/// Считает путь, куда будет писаться entities для kind.
pub fn entity_folder(paths: &OutputPaths, kind: EntityKind) -> std::path::PathBuf {
    paths.output.join("entities").join(kind.folder())
}

/// Удаляет старую папку entities (для --force).
pub fn clean_entities_dir(paths: &OutputPaths) -> Result<()> {
    let dir = paths.output.join("entities");
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    let dump = paths.output.join("entities.jsonl");
    if dump.exists() {
        std::fs::remove_file(dump)?;
    }
    Ok(())
}

// заглушка, чтобы Path не был unused при фиче без тестов
#[allow(dead_code)]
fn _ph(_p: &Path) {}