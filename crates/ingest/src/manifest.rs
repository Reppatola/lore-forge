use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::chapter::Chapter;
use crate::error::Result;

/// Реестр обработанных глав: хеш, ревизия, когда обработано.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub chapters: BTreeMap<u32, ChapterRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterRecord {
    pub number: u32,
    pub title: String,
    pub file: String,
    pub hash: String,
    pub revision: u32,
    pub processed_at: String,
    pub chunks: usize,
    pub entities_created: usize,
    pub entities_updated: usize,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;
        if text.trim().is_empty() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    /// Возвращает главы, которых нет в манифесте или у которых изменился хеш.
    pub fn changed<'a>(&self, chapters: &'a [Chapter]) -> Vec<&'a Chapter> {
        chapters
            .iter()
            .filter(|ch| match self.chapters.get(&ch.number) {
                Some(rec) => rec.hash != ch.hash,
                None => true,
            })
            .collect()
    }

    pub fn record(
        &mut self,
        ch: &Chapter,
        chunks: usize,
        created: usize,
        updated: usize,
    ) {
        let prev_rev = self
            .chapters
            .get(&ch.number)
            .map(|r| r.revision)
            .unwrap_or(0);
        let rec = ChapterRecord {
            number: ch.number,
            title: ch.title.clone(),
            file: ch.path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string(),
            hash: ch.hash.clone(),
            revision: prev_rev + 1,
            processed_at: chrono::Utc::now().to_rfc3339(),
            chunks,
            entities_created: created,
            entities_updated: updated,
        };
        self.chapters.insert(ch.number, rec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn ch(n: u32, h: &str) -> Chapter {
        Chapter {
            number: n,
            title: "t".into(),
            path: PathBuf::from(format!("{:03}_t.md", n)),
            text: String::new(),
            hash: h.into(),
            line_count: 0,
        }
    }

    #[test]
    fn changed_detects_new_and_modified() {
        let mut m = Manifest::default();
        let a = ch(1, "aaa");
        let b = ch(2, "bbb");
        m.chapters.insert(1, ChapterRecord {
            number: 1, title: "t".into(), file: "001_t.md".into(),
            hash: "aaa".into(), revision: 1,
            processed_at: String::new(), chunks: 0,
            entities_created: 0, entities_updated: 0,
        });
        let list = vec![a, b];
        let diff = m.changed(&list);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].number, 2);
    }
}
