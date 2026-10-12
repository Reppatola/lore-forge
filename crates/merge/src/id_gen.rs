use loreforge_core::{slugify, EntityId, EntityKind};

/// Генерирует ID из имени и типа сущности.
/// "Князь Андрей" + Character → "char_knyaz_andrey"
pub fn make_id(kind: EntityKind, primary_name: &str) -> Option<EntityId> {
    let slug = slugify(primary_name);
    if slug.is_empty() {
        return None;
    }
    let id_str = format!("{}_{}", kind.id_prefix(), slug);
    EntityId::new(id_str).ok()
}

/// Ключ группировки: (kind, slug).
/// Используется для слияния записей из разных чанков.
pub fn group_key(kind: EntityKind, primary_name: &str) -> Option<String> {
    let slug = slugify(primary_name);
    if slug.is_empty() {
        return None;
    }
    Some(format!("{}:{}", kind.id_prefix(), slug))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_id() {
        let id = make_id(EntityKind::Character, "Князь Андрей").unwrap();
        assert_eq!(id.as_str(), "char_knyaz_andrey");
    }

    #[test]
    fn group_key_ok() {
        let k = group_key(EntityKind::City, "Петербург").unwrap();
        assert_eq!(k, "city:peterburg");
    }

    #[test]
    fn empty_name() {
        assert!(make_id(EntityKind::Character, "!!!").is_none());
    }
}