use std::collections::BTreeMap;

use chrono::Utc;
use loreforge_core::{Entity, EntityId, EntityKind, Relation, Source};

use crate::collect::CollectedEntity;

/// Сливает группу записей об одной сущности в единый `Entity`.
pub fn merge_group(records: &[CollectedEntity]) -> Entity {
    debug_assert!(!records.is_empty());

    let first = &records[0];
    let kind = first.entity.kind;
    let id: EntityId = first.entity.id.clone();

    // --- names: объединить, dedup, порядок сохранить
    let mut names: Vec<String> = Vec::new();
    for r in records {
        for n in &r.entity.names {
            if !names.iter().any(|x| x == n) {
                names.push(n.clone());
            }
        }
    }

    // --- subtype: первый непустой
    let subtype = records
        .iter()
        .find_map(|r| r.entity.subtype.clone());

    // --- description: самый длинный
    let description = records
        .iter()
        .map(|r| r.entity.description.clone())
        .max_by_key(|d| d.chars().count())
        .unwrap_or_default();

    // --- tags: объединить, dedup
    let mut tags: Vec<String> = Vec::new();
    for r in records {
        for t in &r.entity.tags {
            if !tags.iter().any(|x| x == t) {
                tags.push(t.clone());
            }
        }
    }

    // --- attributes: плоский merge (null из первой заполняется из другой)
    let mut attributes = serde_json::Map::new();
    for r in records {
        for (k, v) in &r.entity.attributes {
            if v.is_null() {
                continue;
            }
            attributes
                .entry(k.clone())
                .or_insert_with(|| v.clone());
        }
    }

    // --- relations: dedup по (target_str, kind)
    let mut relations: Vec<Relation> = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();
    for r in records {
        for rel in &r.entity.relations {
            let key = (
                rel.target.as_str().to_string(),
                format!("{:?}", rel.kind),
            );
            if seen.iter().any(|k| k == &key) {
                continue;
            }
            seen.push(key);
            relations.push(rel.clone());
        }
    }

    // --- events
    let mut events: Vec<EntityId> = Vec::new();
    for r in records {
        for e in &r.entity.events {
            if !events.iter().any(|x| x == e) {
                events.push(e.clone());
            }
        }
    }

    // --- sources: объединить, dedup по (chapter, quote).
    // Одинаковая цитата из разных сцен — оставляем первую.
    let mut sources: Vec<Source> = Vec::new();
    let mut seen_src: Vec<(u32, String)> = Vec::new();
    for r in records {
        for s in &r.entity.sources {
            // Пустую цитату вообще не сохраняем как source, если есть другие
            let key = (s.chapter, s.quote.clone());
            if seen_src.iter().any(|k| k == &key) {
                continue;
            }
            seen_src.push(key);
            sources.push(s.clone());
        }
    }

    // --- meta
    let mut extractors: Vec<String> = Vec::new();
    for r in records {
        let m = &r.model;
        if !m.is_empty() && !extractors.iter().any(|x| x == m) {
            extractors.push(m.clone());
        }
    }
    let extractor_str = extractors.join(",");

    let confidence = {
        let sum: f32 = records.iter().map(|r| r.entity.meta.confidence).sum();
        sum / records.len() as f32
    };

    let mut entity = Entity::new(id, kind);
    entity.subtype = subtype;
    entity.names = names;
    entity.description = description;
    entity.tags = tags;
    entity.attributes = attributes;
    entity.relations = relations;
    entity.events = events;
    entity.sources = sources;
    entity.meta.confidence = confidence;
    entity.meta.extractor = extractor_str;
    entity.meta.revision = records.len() as u32;
    entity.meta.created_at = Utc::now();
    entity.meta.updated_at = Utc::now();

    entity
}

/// Группирует записи по (kind, slug первого имени).
/// Возвращает map: group_key -> Vec<CollectedEntity>.
pub fn group_records(
    records: Vec<CollectedEntity>,
) -> BTreeMap<String, Vec<CollectedEntity>> {
    let mut groups: BTreeMap<String, Vec<CollectedEntity>> = BTreeMap::new();
    for rec in records {
        let Some(first_name) = rec.entity.names.first() else {
            continue; // без имени — отсеиваем
        };
        let kind: EntityKind = rec.entity.kind;
        let slug = loreforge_core::slugify(first_name);
        if slug.is_empty() {
            continue;
        }
        let key = format!("{}:{}", kind.id_prefix(), slug);
        groups.entry(key).or_default().push(rec);
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use loreforge_core::EntityKind;

    fn mk(name: &str, desc: &str, scene: usize) -> CollectedEntity {
        let slug = loreforge_core::slugify(name);
        let id = loreforge_core::EntityId::new(format!("char_{slug}")).unwrap();
        let mut e = Entity::new(id, EntityKind::Character);
        e.names = vec![name.to_string()];
        e.description = desc.to_string();
        let mut src = loreforge_core::Source::with_scene(1, scene);
        src.quote = format!("q{scene}");
        e.sources.push(src);
        CollectedEntity {
            chapter: 1,
            scene_index: scene,
            model: "test".into(),
            entity: e,
        }
    }

    #[test]
    fn merges_descriptions_by_length() {
        let group = vec![
            mk("Андрей", "кратко", 0),
            mk("Андрей", "очень длинное описание", 1),
            mk("Андрей", "средне", 2),
        ];
        let e = merge_group(&group);
        assert_eq!(e.description, "очень длинное описание");
        assert_eq!(e.meta.revision, 3);
        assert_eq!(e.sources.len(), 3);
    }

    #[test]
    fn groups_by_name() {
        let recs = vec![mk("Андрей", "a", 0), mk("Андрей", "b", 1), mk("Лиза", "c", 0)];
        let groups = group_records(recs);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("char:andrey").unwrap().len(), 2);
        assert_eq!(groups.get("char:liza").unwrap().len(), 1);
    }
}