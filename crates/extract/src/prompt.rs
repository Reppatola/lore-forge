use std::path::{Path, PathBuf};

use crate::error::{ExtractError, Result};

/// Шаблон промпта. Читается из файла или берётся встроенный.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    raw: String,
    path: PathBuf,
}

impl PromptTemplate {
    /// Загружает из файла. Если файла нет — ошибка.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        Self::load_or_builtin(path, None)
    }

    /// Загружает из файла, либо использует встроенный дефолт.
    pub fn load_or_builtin(path: impl AsRef<Path>, builtin: Option<&'static str>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let raw = if path.exists() {
            let s = std::fs::read_to_string(&path)?;
            s.trim_start_matches('\u{feff}').to_string()
        } else if let Some(b) = builtin {
            tracing::info!(
                "файл {} не найден, использую встроенный промпт",
                path.display()
            );
            b.to_string()
        } else {
            return Err(ExtractError::PromptNotFound(path.display().to_string()));
        };
        Ok(Self { raw, path })
    }

    pub fn render(&self, scene: &str, prev_context: &str, next_context: &str) -> String {
        let mut scene_full = String::new();

        if !prev_context.trim().is_empty() {
            scene_full.push_str("[КОНТЕКСТ ДО]\n");
            scene_full.push_str(prev_context);
            scene_full.push_str("\n\n");
        }

        scene_full.push_str(scene);

        if !next_context.trim().is_empty() {
            scene_full.push_str("\n\n[КОНТЕКСТ ПОСЛЕ]\n");
            scene_full.push_str(next_context);
        }

        self.raw.replace("{SCENE}", &scene_full)
    }

    pub fn render_strict(&self, scene: &str, prev_context: &str, next_context: &str) -> String {
        let base = self.render(scene, prev_context, next_context);
        format!(
            "КРИТИЧНО: ответ ДОЛЖЕН быть валидным JSON. \
             Начни с `{{` и закончи `}}`. Без markdown, без текста вокруг.\n\n{base}"
        )
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("p.md");
        std::fs::write(&p, "scene: {SCENE}").unwrap();
        let t = PromptTemplate::load(&p).unwrap();
        let r = t.render("AAA", "BEFORE", "AFTER");
        assert!(r.contains("AAA"));
        assert!(r.contains("BEFORE"));
        assert!(r.contains("AFTER"));
    }

    #[test]
    fn builtin_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("nonexistent.md");
        let t = PromptTemplate::load_or_builtin(&p, Some("builtin: {SCENE}")).unwrap();
        assert!(t.render("XXX", "", "").contains("XXX"));
    }
}