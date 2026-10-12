use std::path::Path;
use walkdir::WalkDir;

use crate::chapter::Chapter;
use crate::error::{IngestError, Result};

/// Рекурсивно сканирует папку и загружает все главы.
pub fn scan_input(dir: &Path) -> Result<Vec<Chapter>> {
    if !dir.exists() {
        return Err(IngestError::NoInput(dir.display().to_string()));
    }

    let mut chapters = Vec::new();

    for entry in WalkDir::new(dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "md" && ext != "txt" {
            continue;
        }
        match load_chapter(path) {
            Ok(ch) => chapters.push(ch),
            Err(e) => {
                tracing::warn!("пропущен {}: {e}", path.display());
            }
        }
    }

    chapters.sort_by_key(|c| c.number);
    Ok(chapters)
}

/// Загружает одну главу. Парсит номер и заголовок из имени файла.
pub fn load_chapter(path: &Path) -> Result<Chapter> {
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| IngestError::InvalidFilename(path.display().to_string()))?;

    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| IngestError::InvalidFilename(filename.to_string()))?;

    let (number, title) = parse_filename(stem)?;

    let bytes = std::fs::read(path)?;
    let (text, encoding) = decode_text(&bytes);
    if encoding != "utf-8" {
        tracing::warn!(
            "{}: использован fallback-encoding {}",
            path.display(),
            encoding
        );
    }

    // BOM в начале — обрежем
    let text = text.trim_start_matches('\u{feff}').to_string();

    let hash = blake3::hash(text.as_bytes()).to_hex().to_string();
    let line_count = text.lines().count() as u32;

    Ok(Chapter {
        number,
        title,
        path: path.to_path_buf(),
        text,
        hash,
        line_count,
    })
}

/// Декодирует байты: сначала UTF-8, если не вышло — CP1251.
fn decode_text(bytes: &[u8]) -> (String, &'static str) {
    // убираем BOM
    let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };

    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), "utf-8"),
        Err(_) => {
            // fallback — декодируем байт за байтом через CP1251-таблицу
            (decode_cp1251(bytes), "windows-1251")
        }
    }
}

/// Минимальный CP1251-декодер (без внешних зависимостей).
/// Покрывает кириллицу и ASCII; остальное → '?'.
fn decode_cp1251(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if b < 0x80 {
            out.push(b as char);
        } else if (0xC0..=0xFF).contains(&b) {
            // А-я в CP1251: 0xC0..=0xFF = А..я
            let code = 0x0410 + (b as u32 - 0xC0);
            if let Some(c) = char::from_u32(code) {
                out.push(c);
            } else {
                out.push('?');
            }
        } else if b == 0xA8 {
            out.push('Ё');
        } else if b == 0xB8 {
            out.push('ё');
        } else {
            // прочие символы CP1251 пропускаем (можно расширить)
            out.push('?');
        }
    }
    out
}

fn parse_filename(stem: &str) -> Result<(u32, String)> {
    let digits_end = stem
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit())
        .last()
        .map(|(i, _)| i + 1)
        .unwrap_or(0);

    if digits_end == 0 {
        return Err(IngestError::InvalidFilename(format!(
            "файл должен начинаться с порядкового номера: {}",
            stem
        )));
    }

    let number: u32 = stem[..digits_end]
        .parse()
        .map_err(|_| IngestError::InvalidFilename(stem.to_string()))?;

    let rest = stem[digits_end..].trim_start_matches(['_', '-', ' ']);
    let title = if rest.is_empty() {
        format!("Глава {}", number)
    } else {
        rest.replace('_', " ")
    };

    Ok((number, title))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_filename_ok() {
        let (n, t) = parse_filename("001_пробуждение").unwrap();
        assert_eq!(n, 1);
        assert_eq!(t, "пробуждение");
    }

    #[test]
    fn parse_filename_no_number() {
        assert!(parse_filename("prologue").is_err());
    }

    #[test]
    fn cp1251_fallback() {
        // "Привет" в CP1251 = CF F0 E8 E2 E5 F2
        let bytes = [0xCF, 0xF0, 0xE8, 0xE2, 0xE5, 0xF2];
        assert_eq!(decode_cp1251(&bytes), "Привет");
    }

    #[test]
    fn utf8_ok() {
        let bytes = "Привет".as_bytes();
        let (s, enc) = decode_text(bytes);
        assert_eq!(s, "Привет");
        assert_eq!(enc, "utf-8");
    }

    #[test]
    fn bom_stripped() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("Привет".as_bytes());
        let (s, _) = decode_text(&bytes);
        assert_eq!(s, "Привет");
    }
}
