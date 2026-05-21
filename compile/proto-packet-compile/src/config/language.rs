use serde::{Deserialize, Serialize};

/// A target language for code generation.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "rust")]
    Rust,
}
