use crate::error::{ExtractError, Result};
use crate::types::LlmResponse;

/// Парсит ответ LLM: срезает markdown-обёртки, вырезает JSON-объект,
/// парсит в LlmResponse.
pub fn parse_llm_response(raw: &str) -> Result<LlmResponse> {
    let cleaned = strip_markdown(raw);
    let json_slice = extract_json_object(&cleaned).ok_or_else(|| {
        ExtractError::InvalidLlmResponse {
            attempts: 0,
            reason: format!("не найден JSON-объект в ответе (первые 300 символов): {}", preview(&cleaned, 300)),
        }
    })?;

    serde_json::from_str::<LlmResponse>(json_slice).map_err(|e| {
        ExtractError::InvalidLlmResponse {
            attempts: 0,
            reason: format!("serde error: {e}\nJSON: {}", preview(json_slice, 500)),
        }
    })
}

/// Убирает ```json ... ``` обёртки, если есть.
fn strip_markdown(s: &str) -> String {
    let trimmed = s.trim();

    // ```json\n...\n```
    if let Some(rest) = trimmed.strip_prefix("```json") {
        return rest.trim_start_matches('\n').trim_end_matches("```").trim().to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        return rest.trim_start_matches('\n').trim_end_matches("```").trim().to_string();
    }
    trimmed.to_string()
}

/// Находит первый сбалансированный JSON-объект в строке.
fn extract_json_object(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let start = s.find('{')?;

    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;

    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }

        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[start..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

fn preview(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_json() {
        let raw = r#"{"entities":[],"unresolved":[]}"#;
        let r = parse_llm_response(raw).unwrap();
        assert!(r.entities.is_empty());
    }

    #[test]
    fn markdown_json() {
        let raw = "```json\n{\"entities\":[],\"unresolved\":[]}\n```";
        let r = parse_llm_response(raw).unwrap();
        assert!(r.entities.is_empty());
    }

    #[test]
    fn text_before_json() {
        let raw = "Вот результат:\n{\"entities\":[],\"unresolved\":[]}\nСпасибо!";
        let r = parse_llm_response(raw).unwrap();
        assert!(r.entities.is_empty());
    }

    #[test]
    fn with_entities() {
        let raw = r#"{"entities":[{"type":"character","names":["Иван"]}],"unresolved":[]}"#;
        let r = parse_llm_response(raw).unwrap();
        assert_eq!(r.entities.len(), 1);
        assert_eq!(r.entities[0].kind, "character");
    }
}
