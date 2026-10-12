use loreforge_core::{Entity, EntityId, EntityKind, slugify, Source};

use crate::error::{ExtractError, Result};
use crate::types::ExtractedEntity;

/// Маппит строковый тип LLM → EntityKind из core.
pub fn parse_kind(s: &str) -> Result<EntityKind> {
    Ok(match s.to_lowercase().as_str() {
        "character" => EntityKind::Character,
        "location"  => EntityKind::Location,
        "city"      => EntityKind::City,
        "village"   => EntityKind::Village,
        "base"      => EntityKind::Base,
        "water"     => EntityKind::Water,
        "building"  => EntityKind::Building,
        "quest"     => EntityKind::Quest,
        "faction"   => EntityKind::Faction,
        "clan"      => EntityKind::Clan,
        "alliance"  => EntityKind::Alliance,
        "event"     => EntityKind::Event,
        "item"      => EntityKind::Item,
        "creature"  => EntityKind::Creature,
        "resource"  => EntityKind::Resource,
        other => return Err(ExtractError::InvalidLlmResponse {
            attempts: 0,
            reason: format!("неизвестный тип сущности: {other}"),
        }),
    })
}

/// Собирает core::Entity из сырого ExtractedEntity.
/// ID генерируется автоматически: `<prefix>_<slug>`.
pub fn validate_and_build(
    raw: &ExtractedEntity,
    chapter: u32,
    model: &str,
    confidence: f32,
) -> Result<Entity> {
    let kind = parse_kind(&raw.kind)?;

    let primary_name = raw
        .names
        .first()
        .cloned()
        .unwrap_or_else(|| format!("unnamed_{}", slugify(&raw.kind)));

    let slug = slugify(&primary_name);
    if slug.is_empty() {
        return Err(ExtractError::InvalidLlmResponse {
            attempts: 0,
            reason: format!("пустой slug из имени: {primary_name:?}"),
        });
    }
    let id_str = format!("{}_{}", kind.id_prefix(), slug);
    let id = EntityId::new(id_str.clone())?;

    let mut entity = Entity::new(id, kind);

    entity.subtype = raw.subtype.clone();
    entity.names = raw.names.clone();
    entity.description = raw.description.clone();
    entity.tags = raw.tags.clone();
    entity.attributes = raw.attributes.clone();

    // Источники: цитаты + глава
    for quote in &raw.quotes {
        let mut src = Source::new(chapter);
        src.quote = quote.clone();
        entity.sources.push(src);
    }
    if entity.sources.is_empty() {
        entity.sources.push(Source::new(chapter));
    }

    entity.meta.extractor = model.to_string();
    entity.meta.confidence = confidence;

    Ok(entity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_parsing() {
        assert_eq!(parse_kind("character").unwrap(), EntityKind::Character);
        assert_eq!(parse_kind("FACTION").unwrap(), EntityKind::Faction);
        assert!(parse_kind("unknown").is_err());
    }

    #[test]
    fn build_entity() {
        let raw = ExtractedEntity {
            kind: "character".into(),
            subtype: Some("npc".into()),
            names: vec!["Князь Андрей".into()],
            description: "князь".into(),
            tags: vec!["военный".into()],
            attributes: serde_json::Map::new(),
            relations: vec![],
            quotes: vec!["Через двадцать минут он встанет".into()],
        };
        let e = validate_and_build(&raw, 1, "test", 0.9).unwrap();
        assert_eq!(e.kind, EntityKind::Character);
        assert_eq!(e.id.as_str(), "char_knyaz_andrey");
        assert_eq!(e.subtype.as_deref(), Some("npc"));
        assert_eq!(e.sources.len(), 1);
        assert_eq!(e.meta.extractor, "test");
    }
}
