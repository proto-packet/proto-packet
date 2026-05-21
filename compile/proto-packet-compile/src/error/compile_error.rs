use clerr::{Code, Report};
use colored::{ColoredString, Colorize};
use file_storage::FolderPath;
use proto_packet_link::LinkError;
use std::fmt::{Display, Formatter};

/// An error compiling a project.
#[derive(Debug)]
pub enum CompileError {
    /// An error reading the config file.
    ReadConfig(file_storage::Error),

    /// An error parsing the JSON config file.
    ParseJsonConfig(serde_json::Error),

    /// An error parsing the YAML config file.
    ParseYamlConfig(serde_yaml::Error),

    /// An error reading a schema file.
    ReadSchema(file_storage::Error),

    /// An error parsing a schema file.
    ParseSchema(Report),

    /// An error validating a schema file.
    ValidateSchema(Report),

    /// An error linking a schema file.
    LinkSchema(LinkError),

    /// Two declared deps share the same `root`.
    DuplicateDepRoot { root: String },

    /// A declared dep's `root` collides with a top-level mod in the local project.
    DepRootShadowsMod { root: String },

    /// todo -- ?
    InvalidModPath { root: FolderPath, file_name: String },

    /// An error cleaning the target folder.
    CleanTarget(file_storage::Error),

    /// An error generating source code.
    GenSource(Report),

    /// todo -- ?
    InvalidSourceFileName { root: FolderPath, file_name: String },

    /// An error writing source code.
    WriteSource(file_storage::Error),
}

impl CompileError {
    //! Report

    /// The error code prefix.
    const ERROR_CODE_PREFIX: &'static str = "C";

    /// Generates the report.
    ///
    /// # Note
    /// Do not call for variants with their own report.
    fn report(&self) -> Report {
        Report::from(Code::error(self.code(), self.message())).with_entry(self.entry())
    }

    /// Generates the report code.
    ///
    /// # Note
    /// Do not call for variants with their own report.
    fn code(&self) -> String {
        let code: &str = match self {
            CompileError::ReadConfig(_) => "READ_CONFIG",
            CompileError::ParseJsonConfig(_) => "PARSE_CONFIG_JSON",
            CompileError::ParseYamlConfig(_) => "PARSE_CONFIG_YAML",
            CompileError::ReadSchema(_) => "READ_SCHEMA",
            CompileError::ParseSchema(_) => unreachable!(),
            CompileError::ValidateSchema(_) => unreachable!(),
            CompileError::LinkSchema(_) => "LINK_SCHEMA",
            CompileError::DuplicateDepRoot { .. } => "DUPLICATE_DEP_ROOT",
            CompileError::DepRootShadowsMod { .. } => "DEP_ROOT_SHADOWS_MOD",
            CompileError::InvalidModPath { .. } => "INVALID_MOD_PATH",
            CompileError::CleanTarget(_) => "CLEAN_TARGET",
            CompileError::GenSource(_) => unreachable!(),
            CompileError::InvalidSourceFileName { .. } => "INVALID_SOURCE_FILE_NAME",
            CompileError::WriteSource(_) => "WRITE_SOURCE",
        };
        format!("{}_{}", Self::ERROR_CODE_PREFIX, code)
    }

    /// Generates the report message.
    ///
    /// # Note
    /// Do not call for variants with their own report.
    fn message(&self) -> String {
        match self {
            CompileError::ReadConfig(_error) => "error reading config",
            CompileError::ParseJsonConfig(_error) => "error parsing JSON config",
            CompileError::ParseYamlConfig(_error) => "error parsing YAML config",
            CompileError::ReadSchema(_error) => "error reading schema file",
            CompileError::ParseSchema(_report) => unreachable!(),
            CompileError::ValidateSchema(_report) => unreachable!(),
            CompileError::LinkSchema(_error) => "error linking schema",
            CompileError::DuplicateDepRoot { .. } => "two deps share the same root",
            CompileError::DepRootShadowsMod { .. } => "dep root shadows a local top-level mod",
            CompileError::InvalidModPath { .. } => todo!(),
            CompileError::CleanTarget(_error) => "error cleaning target folder",
            CompileError::GenSource(_report) => unreachable!(),
            CompileError::InvalidSourceFileName { .. } => todo!(),
            CompileError::WriteSource(_error) => "error writing source file",
        }
        .to_string()
    }
    /// Generates the report entry.
    ///
    /// # Note
    /// Do not call for variants with their own report.
    fn entry(&self) -> Vec<ColoredString> {
        let s: String = match self {
            CompileError::ReadConfig(error) => error.to_string(),
            CompileError::ParseJsonConfig(error) => error.to_string(),
            CompileError::ParseYamlConfig(error) => error.to_string(),
            CompileError::ReadSchema(error) => error.to_string(),
            CompileError::ParseSchema(_) => unreachable!(),
            CompileError::ValidateSchema(_) => unreachable!(),
            CompileError::LinkSchema(error) => error.to_string(),
            CompileError::DuplicateDepRoot { root } => format!("dep root '{}'", root),
            CompileError::DepRootShadowsMod { root } => format!("dep root '{}'", root),
            CompileError::InvalidModPath { .. } => todo!(),
            CompileError::CleanTarget(error) => error.to_string(),
            CompileError::GenSource(_) => unreachable!(),
            CompileError::InvalidSourceFileName { .. } => todo!(),
            CompileError::WriteSource(error) => error.to_string(),
        };
        vec![s.normal()]
    }
}

impl From<CompileError> for Report {
    fn from(error: CompileError) -> Self {
        match error {
            CompileError::ParseSchema(report)
            | CompileError::ValidateSchema(report)
            | CompileError::GenSource(report) => report,
            _ => error.report(),
        }
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                CompileError::ParseSchema(report)
                | CompileError::ValidateSchema(report)
                | CompileError::GenSource(report) => report.to_string(),
                _ => self.report().to_string(),
            }
        )
    }
}

impl std::error::Error for CompileError {}
