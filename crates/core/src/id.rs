use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

/// Стабильный идентификатор сущности.
/// Формат: `<prefix>_<slug>[_<n>]`, например `char_aragorn`, `city_novgorod`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(String);

impl EntityId {
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s = raw.into();
        if s.is_empty()
            || !s
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(CoreError::InvalidId(s));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Транслитерация русского в латиницу + slug-нормализация.
/// "Арагорн" -> "aragorn", "Северный Лес" -> "severnyy_les"
/// "Князь Андрей" -> "knyaz_andrey", "Петербург" -> "peterburg"
pub fn slugify(input: &str) -> String {
    let translit = |c: char| -> &'static str {
        match c {
            'а' => "a",  'б' => "b",  'в' => "v",  'г' => "g",
            'д' => "d",  'е' => "e",  'ё' => "e",  'ж' => "zh",
            'з' => "z",  'и' => "i",  'й' => "y",  'к' => "k",
            'л' => "l",  'м' => "m",  'н' => "n",  'о' => "o",
            'п' => "p",  'р' => "r",  'с' => "s",  'т' => "t",
            'у' => "u",  'ф' => "f",  'х' => "h",  'ц' => "ts",
            'ч' => "ch", 'ш' => "sh", 'щ' => "sch",'ъ' => "",
            'ы' => "y",  'ь' => "",   'э' => "e",  'ю' => "yu",
            'я' => "ya",
            _ => "",
        }
    };

    let mut out = String::new();
    let mut prev_underscore = false;

    for orig in input.chars() {
        let c = orig.to_lowercase().next().unwrap_or(orig);

        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_underscore = false;
        } else if c.is_alphabetic() {
            let t = translit(c);
            for ch in t.chars() {
                out.push(ch);
            }
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }

    out.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_latin() {
        assert_eq!(slugify("Aragorn"), "aragorn");
    }

    #[test]
    fn slugify_cyrillic() {
        assert_eq!(slugify("Арагорн"), "aragorn");
    }

    #[test]
    fn slugify_spaces() {
        assert_eq!(slugify("Северный Лес"), "severnyy_les");
    }

    #[test]
    fn slugify_knyaz() {
        assert_eq!(slugify("Князь Андрей"), "knyaz_andrey");
    }

    #[test]
    fn slugify_city() {
        assert_eq!(slugify("Петербург"), "peterburg");
    }

    #[test]
    fn slugify_empty() {
        assert_eq!(slugify("!!!"), "");
    }
}