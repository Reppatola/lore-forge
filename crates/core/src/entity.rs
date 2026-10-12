use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::id::EntityId;
use crate::relation::Relation;
use crate::source::Source;

/// Тип сущности (верхний уровень таксономии).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Character,
    Location,
    City,
    Village,
    Base,
    Water,
    Building,
    Quest,
    Faction,
    Clan,
    Alliance,
    Event,
    Item,
    Creature,
    Resource,
}

impl EntityKind {
    /// Префикс для ID: char_, loc_, city_, ...
    pub fn id_prefix(self) -> &'static str {
        match self {
            Self::Character => "char",
            Self::Location  => "loc",
            Self::City      => "city",
            Self::Village   => "vill",
            Self::Base      => "base",
            Self::Water     => "water",
            Self::Building  => "bld",
            Self::Quest     => "quest",
            Self::Faction   => "fac",
            Self::Clan      => "clan",
            Self::Alliance  => "ally",
            Self::Event     => "event",
            Self::Item      => "item",
            Self::Creature  => "crea",
            Self::Resource  => "res",
        }
    }

    /// Имя папки в output.
    pub fn folder(self) -> &'static str {
        match self {
            Self::Character => "characters",
            Self::Location  => "locations",
            Self::City      => "cities",
            Self::Village   => "villages",
            Self::Base      => "bases",
            Self::Water     => "waters",
            Self::Building  => "buildings",
            Self::Quest     => "quests",
            Self::Faction   => "factions",
            Self::Clan      => "clans",
            Self::Alliance  => "alliances",
            Self::Event     => "events",
            Self::Item      => "items",
            Self::Creature  => "creatures",
            Self::Resource  => "resources",
        }
    }
}

/// Свободные атрибуты под каждый тип.
/// Хранятся как JSON-объект, чтобы не плодить структуры.
pub type EntityAttributes = serde_json::Map<String, serde_json::Value>;

/// Метаданные объекта.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMeta {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 0.0..1.0 — уверенность экстрактора.
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    /// Имя модели/экстрактора.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub extractor: String,
    /// Номер ревизии (растёт с каждым обогащением).
    #[serde(default = "default_revision")]
    pub revision: u32,
}

fn default_confidence() -> f32 { 1.0 }
fn default_revision()   -> u32 { 1 }

/// Задел под генерацию картинок (позже).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityVisual {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
}

/// Упоминание объекта в главе.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appearance {
    pub chapter: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub quote: String,
}

/// Главный объект базы.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    #[serde(rename = "type")]
    pub kind: EntityKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,

    /// Все имена и алиасы, включая основное.
    #[serde(default)]
    pub names: Vec<String>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Тип-специфичные поля.
    #[serde(default)]
    pub attributes: EntityAttributes,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<Relation>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub appearances: Vec<Appearance>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<EntityId>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timeline: Vec<serde_json::Value>,

    #[serde(default)]
    pub visual: EntityVisual,

    /// Свободные поля.
    #[serde(default)]
    pub extra: serde_json::Map<String, serde_json::Value>,

    /// Все источники (цитаты из глав).
    #[serde(default)]
    pub sources: Vec<Source>,

    pub meta: EntityMeta,
}

impl Entity {
    pub fn new(id: EntityId, kind: EntityKind) -> Self {
        let now = Utc::now();
        Self {
            id,
            kind,
            subtype: None,
            names: Vec::new(),
            description: String::new(),
            tags: Vec::new(),
            status: None,
            attributes: EntityAttributes::new(),
            relations: Vec::new(),
            appearances: Vec::new(),
            events: Vec::new(),
            timeline: Vec::new(),
            visual: EntityVisual::default(),
            extra: serde_json::Map::new(),
            sources: Vec::new(),
            meta: EntityMeta {
                created_at: now,
                updated_at: now,
                confidence: 1.0,
                extractor: String::new(),
                revision: 1,
            },
        }
    }
}