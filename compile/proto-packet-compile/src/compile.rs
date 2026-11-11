use crate::report::path_report;
use crate::{Config, SchemaSource, check_sources, read_schema_sources};
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
    //! Constants

    /// The target languages.
    pub const ALL: [Self; 1] = [Self::Rust];
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
        Self::ALL
            .into_iter()
            .find(|language| language.name() == name)
    }
}

/// Compiles the project in the `source` folder to `language` code in the `target` folder.
///
/// The code is generated in memory first. Only if that succeeds are the generated files written to
/// the `target` folder, by the target mode of the config. A `target` folder inside the `source`
/// folder is fine, as long as it has no schema files.
pub fn compile(language: Language, source: &Path, target: &Path) -> Result<(), Report> {
    check_target(source, target)?;
    let config: Config = Config::read(source)?;
    let sources: Vec<SchemaSource> = read_schema_sources(source, &config)?;
    check_schema_files(&sources, target)?;
    let project: Project = check_sources(&sources)?;
    let files: Vec<GeneratedFile> = generate(language, &project)?;
    config.target_mode.write(target, &files)
}

/// Generates the `language` code of the linked `project`, in memory.
pub(crate) fn generate(
    language: Language,
    project: &Project,
) -> Result<Vec<GeneratedFile>, Report> {
    match language {
        Language::Rust => proto_packet_gen::rust::generate(project),
    }
    .map_err(|error| Report::from(Code::error(error.code(), error.to_string())))
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

/// Checks the `target` folder holds none of the schema `sources`, since it is about to be emptied.
fn check_schema_files(sources: &[SchemaSource], target: &Path) -> Result<(), Report> {
    let Ok(canonical_target) = std::fs::canonicalize(target) else {
        return Ok(());
    };
    for source in sources {
        if let Ok(path) = std::fs::canonicalize(&source.path)
            && path.starts_with(&canonical_target)
        {
            let properties: Properties = Properties::default()
                .with_property("schema", path.display().to_string())
                .with_property("target", canonical_target.display().to_string());
            return Err(Report::from(Code::error(
                "C_TARGET_CONTAINS_SCHEMA",
                "the target folder would delete a schema file",
            ))
            .with_entry(properties));
        }
    }
    Ok(())
}
