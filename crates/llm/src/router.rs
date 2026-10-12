use crate::config::LlmConfig;

/// Роутер: имя задачи → имя модели.
/// Задачи:
///   extract_heavy  — 14b, для сложных сцен
///   extract_light  — 9b, повседневное извлечение
///   extract_long   — 64k контекст
///   classify       — быстрая, для тегов
///   code           — coder-модель для промптов/схем
#[derive(Debug, Clone)]
pub struct ModelRouter {
    cfg: LlmConfig,
}

impl ModelRouter {
    pub fn new(cfg: LlmConfig) -> Self {
        Self { cfg }
    }

    pub fn model(&self, task: &str) -> String {
        self.cfg.model_for(task)
    }

    pub fn fallback(&self) -> String {
        self.cfg.default_model.clone()
    }

    /// Список всех задач для отображения в CLI.
    pub fn tasks(&self) -> Vec<(String, String)> {
        self.cfg
            .tasks
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
