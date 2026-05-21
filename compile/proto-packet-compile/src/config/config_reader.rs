use crate::CompileError;
use crate::CompileError::{ParseJsonConfig, ParseYamlConfig, ReadConfig};
use crate::config::Config;
use file_storage::{FilePath, FolderPath};

/// Responsible for reading project configs.
#[derive(Copy, Clone, Debug, Default)]
pub struct ConfigReader {
    _nothing: (),
}

impl ConfigReader {
    //! Read Config

    /// Reads the config for the `source` project root folder.
    pub fn read_config(&self, source: &FolderPath) -> Result<Config, CompileError> {
        let file: FilePath = source
            .clone_append("config.json")
            .to_file()
            .map_err(ReadConfig)?;
        if let Some(content) = file.read_as_string_if_exists().map_err(ReadConfig)? {
            Ok(serde_json::from_str(content.as_str()).map_err(ParseJsonConfig)?)
        } else {
            let file: FilePath = source
                .clone_append("config.yml")
                .to_file()
                .map_err(ReadConfig)?;
            if let Some(content) = file.read_as_string_if_exists().map_err(ReadConfig)? {
                Ok(serde_yaml::from_str(content.as_str()).map_err(ParseYamlConfig)?)
            } else {
                Ok(Config::default())
            }
        }
    }
}
