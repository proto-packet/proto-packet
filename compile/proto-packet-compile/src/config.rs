use crate::report::path_report;
use clerr::{Code, Properties, Report};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// A project config, from the `config.json` or `config.yml` in the project's source folder.
#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// The schema file extension. (default: `.pps`)
    pub schema_extension: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_extension: ".pps".to_string(),
        }
    }
}

impl Config {
    //! Reading

    /// Reads the config of the project in the `source` folder.
    ///
    /// Returns the default config if there is no config file.
    pub fn read(source: &Path) -> Result<Self, Report> {
        let json: PathBuf = source.join("config.json");
        let yaml: PathBuf = source.join("config.yml");
        match (json.is_file(), yaml.is_file()) {
            (true, true) => Err(Report::from(Code::error(
                "C_DUPLICATE_CONFIG",
                "the project has both a `config.json` & a `config.yml`",
            ))
            .with_entry(Properties::default().with_property("path", source.display().to_string()))),
            (true, false) => serde_json::from_str(&read(&json)?)
                .map_err(|error| path_report("C_PARSE_CONFIG", "invalid config", &json, error)),
            (false, true) => serde_yaml::from_str(&read(&yaml)?)
                .map_err(|error| path_report("C_PARSE_CONFIG", "invalid config", &yaml, error)),
            (false, false) => Ok(Self::default()),
        }
    }
}

/// Reads the config file at the `path`.
fn read(path: &Path) -> Result<String, Report> {
    std::fs::read_to_string(path)
        .map_err(|error| path_report("C_READ_CONFIG", "could not read the config", path, error))
}
