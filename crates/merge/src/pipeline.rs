use std::collections::BTreeSet;
use std::time::Instant;

use loreforge_core::Entity;
use loreforge_storage::OutputPaths;

use crate::collect::collect_all;
use crate::entity_merge::{group_records, merge_group};
use crate::error::Result;
use crate::unresolved::collect_unresolved;
use crate::writer;

/// Статистика merge.
#[derive(Debug, Clone, Default)]
pub struct MergeStats {
    pub chapters: usize,
    pub records_in: usize,
    pub entities_unique: usize,
    pub unresolved_count: usize,
    pub elapsed_secs: f64,
}

/// Полный пайплайн: собрать → сгруппировать → слить → записать.
pub fn merge_all(paths: &OutputPaths, force: bool) -> Result<MergeStats> {
    let t0 = Instant::now();

    let extracted_dir = paths.output.join("extracted");
    let records = collect_all(&extracted_dir)?;

    // уникальные главы
    let mut chapters: Vec<u32> = records.iter().map(|r| r.chapter).collect();
    chapters.sort();
    chapters.dedup();

    let records_in = records.len();

    // группировка и слияние
    let groups = group_records(records);
    let mut entities: Vec<Entity> = groups.values().map(|g| merge_group(g)).collect();

    // стабильный порядок: по kind, потом по id
    entities.sort_by(|a, b| {
        let k = format!("{:?}", a.kind).cmp(&format!("{:?}", b.kind));
        if k.is_eq() {
            a.id.as_str().cmp(b.id.as_str())
        } else {
            k
        }
    });

    // имена всех сущностей — для фильтра unresolved
    let mut known_names: BTreeSet<String> = BTreeSet::new();
    for e in &entities {
        for n in &e.names {
            known_names.insert(n.clone());
        }
    }

    let unresolved = collect_unresolved(&extracted_dir, &known_names)?;
    let unresolved_count = unresolved.len();

    // запись
    if force {
        writer::clean_entities_dir(paths)?;
    }

    for e in &entities {
        writer::write_entity(paths, e)?;
    }
    writer::write_jsonl_dump(paths, &entities)?;
    writer::write_index(paths, &entities, &chapters, unresolved_count)?;
    writer::write_unresolved(paths, &unresolved)?;

    Ok(MergeStats {
        chapters: chapters.len(),
        records_in,
        entities_unique: entities.len(),
        unresolved_count,
        elapsed_secs: t0.elapsed().as_secs_f64(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_merge() {
        // Проверяем, что функции компилируются и MergeStats Default работает
        let s = MergeStats::default();
        assert_eq!(s.entities_unique, 0);
    }
}