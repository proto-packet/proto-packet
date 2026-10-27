use crate::report::path_report;
use crate::{Config, SchemaSource, link, read_schema_sources, validate_project};
use clerr::{Code, Properties, Report};
use proto_packet_gen::GeneratedFile;
use proto_packet_tree::Project;
use std::path::{Path, PathBuf};

/// A target language.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Language {
    /// The Rust programming language.
    Rust,
}

impl Language {
    //! Names

    /// Gets the name. (ex: `rust`)
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
        }
    }

    /// Gets the optional language with the `name`.
    pub fn from_name(name: &str) -> Option<Self> {
        [Self::Rust]
            .into_iter()
            .find(|language| language.name() == name)
    }
}

/// Compiles the project in the `source` folder to `language` code in the `target` folder.
///
/// The code is generated in memory first. Only if that succeeds is the `target` folder deleted &
/// the generated files written to it.
pub fn compile(language: Language, source: &Path, target: &Path) -> Result<(), Report> {
    check_target(source, target)?;
    let config: Config = Config::read(source)?;
    let sources: Vec<SchemaSource> = read_schema_sources(source, &config)?;
    let project: Project = validate_project(&sources)?;
    let project: Project = link(&project, &sources)?;
    let files: Vec<GeneratedFile> = match language {
        Language::Rust => proto_packet_gen::rust::generate(&project),
    }
    .map_err(|error| Report::from(Code::error(error.code(), error.to_string())))?;
    write_files(target, &files)
}

/// Checks the `target` folder can be replaced without deleting the `source` folder.
fn check_target(source: &Path, target: &Path) -> Result<(), Report> {
    let source: PathBuf = std::fs::canonicalize(source).map_err(|error| {
        path_report("C_READ_FOLDER", "could not read the folder", source, error)
    })?;
    if let Ok(canonical_target) = std::fs::canonicalize(target)
        && source.starts_with(&canonical_target)
    {
        let properties: Properties = Properties::default()
            .with_property("source", source.display().to_string())
            .with_property("target", canonical_target.display().to_string());
        return Err(Report::from(Code::error(
            "C_TARGET_CONTAINS_SOURCE",
            "the target folder would delete the source folder",
        ))
        .with_entry(properties));
    }
    Ok(())
}

/// Replaces the `target` folder with the generated `files`.
fn write_files(target: &Path, files: &[GeneratedFile]) -> Result<(), Report> {
    let write_error = |path: &Path, error| {
        path_report("C_WRITE_TARGET", "could not write the target", path, error)
    };
    if target.exists() {
        std::fs::remove_dir_all(target).map_err(|error| write_error(target, error))?;
    }
    for file in files {
        let path: PathBuf = target.join(&file.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| write_error(parent, error))?;
        }
        std::fs::write(&path, &file.code).map_err(|error| write_error(&path, error))?;
    }
    Ok(())
}
