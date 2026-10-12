//! Extract: извлечение сущностей из текста через LLM.

pub mod error;
pub mod parser;
pub mod pipeline;
pub mod prompt;
pub mod types;
pub mod validator;

pub use error::{ExtractError, Result};
pub use parser::parse_llm_response;
pub use pipeline::{
    extract_chapter, extract_chunk, ChunkResult, ExtractConfig, ExtractStats,
};
pub use prompt::PromptTemplate;
pub use types::{ChunkExtraction, ExtractedEntity, LlmResponse, UnresolvedMention};
pub use validator::validate_and_build;
