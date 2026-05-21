use serde::{Deserialize, Serialize};

/// A compiler config.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_schema_extension")]
    pub schema_extension: String,
}

fn default_schema_extension() -> String {
    ".pps".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_extension: default_schema_extension(),
        }
    }
}
