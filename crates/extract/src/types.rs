use serde::{Deserialize, Deserializer, Serialize};

/// Хелпер: если поле пришло как `null`, заменить на `Default::default()`.
fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let opt = Option::<T>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Полный ответ LLM по одному чанку.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    #[serde(default, deserialize_with = "null_as_default")]
    pub entities: Vec<ExtractedEntity>,

    #[serde(default, deserialize_with = "null_as_default")]
    pub unresolved: Vec<UnresolvedMention>,
}

/// Сущность как её вернула LLM (до маппинга в core::Entity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    #[serde(rename = "type")]
    pub kind: String,

    #[serde(default)]
    pub subtype: Option<String>,

    #[serde(default, deserialize_with = "null_as_default")]
    pub names: Vec<String>,

    #[serde(default, deserialize_with = "null_as_default")]
    pub description: String,

    #[serde(default, deserialize_with = "null_as_default")]
    pub tags: Vec<String>,

    #[serde(default, deserialize_with = "null_as_default")]
    pub attributes: serde_json::Map<String, serde_json::Value>,

    #[serde(default, deserialize_with = "null_as_default")]
    pub relations: Vec<ExtractedRelation>,

    #[serde(default, deserialize_with = "null_as_default")]
    pub quotes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelation {
    #[serde(default, deserialize_with = "null_as_default")]
    pub target: String,

    #[serde(rename = "kind", default, deserialize_with = "null_as_default")]
    pub kind: String,

    #[serde(default)]
    pub since: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedMention {
    #[serde(default, deserialize_with = "null_as_default")]
    pub mention: String,

    #[serde(default, deserialize_with = "null_as_default")]
    pub reason: String,
}

/// Результат обработки одного чанка.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkExtraction {
    pub chapter: u32,
    pub scene_index: usize,
    pub model: String,
    pub elapsed_secs: f64,
    pub tokens_out: Option<u32>,
    pub tokens_per_sec: Option<f64>,
    pub raw_response: String,
    pub parsed: LlmResponse,
    pub from_cache: bool,
    pub attempts: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_entities() {
        let raw = r#"{"entities": null, "unresolved": null}"#;
        let r: LlmResponse = serde_json::from_str(raw).unwrap();
        assert!(r.entities.is_empty());
        assert!(r.unresolved.is_empty());
    }

    #[test]
    fn null_fields_in_entity() {
        let raw = r#"{
            "type": "character",
            "names": ["Иван"],
            "description": null,
            "tags": null,
            "attributes": null,
            "relations": null,
            "quotes": null
        }"#;
        let e: ExtractedEntity = serde_json::from_str(raw).unwrap();
        assert_eq!(e.names, vec!["Иван"]);
        assert_eq!(e.description, "");
        assert!(e.tags.is_empty());
    }

    #[test]
    fn missing_fields_ok() {
        let raw = r#"{"type": "location", "names": ["Москва"]}"#;
        let e: ExtractedEntity = serde_json::from_str(raw).unwrap();
        assert_eq!(e.kind, "location");
        assert_eq!(e.names, vec!["Москва"]);
    }
}