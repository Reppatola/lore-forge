//! LLM: адаптер Ollama, роутер моделей, кеш, парсер JSON-ответов.

pub mod cache;
pub mod client;
pub mod config;
pub mod error;
pub mod router;
pub mod types;

pub use cache::LlmCache;
pub use client::{ModelInfo, OllamaClient};
pub use config::LlmConfig;
pub use error::{LlmError, Result};
pub use router::ModelRouter;
pub use types::{GenerateOptions, GenerateRequest, GenerateResponse};