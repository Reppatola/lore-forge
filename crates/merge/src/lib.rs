//! Merge: слияние extracted-записей в единые объекты базы.

pub mod collect;
pub mod entity_merge;
pub mod error;
pub mod id_gen;
pub mod pipeline;
pub mod unresolved;
pub mod writer;

pub use error::{MergeError, Result};
pub use pipeline::{merge_all, MergeStats};
