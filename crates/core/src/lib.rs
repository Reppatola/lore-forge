//! РЇРґСЂРѕ lore-forge: Р±Р°Р·РѕРІС‹Рµ С‚РёРїС‹, РёРґРµРЅС‚РёС„РёРєР°С‚РѕСЂС‹, СЃСѓС‰РЅРѕСЃС‚Рё, СЃРІСЏР·Рё.

pub mod entity;
pub mod id;
pub mod relation;
pub mod source;
pub mod error;

pub use entity::{Entity, EntityAttributes, EntityKind, EntityMeta, EntityVisual};
pub use id::{slugify, EntityId};
pub use relation::{Relation, RelationKind};
pub use source::Source;
pub use error::{CoreError, Result};
