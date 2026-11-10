use crate::compile::generate;
use crate::{Config, Language, SchemaSource, link, read_schema_sources, validate_project};
use clerr::Report;
use proto_packet_tree::Project;
use std::path::Path;

/// Checks the project in the source folder for every error that compiling it would find.
///
/// # Note
/// - Nothing is written.
/// - The code of each language is generated in memory, since generating code has its own errors.
pub fn check(source: &Path) -> Result<Project, Report> {
    let config: Config = Config::read(source)?;
    let sources: Vec<SchemaSource> = read_schema_sources(source, &config)?;
    let project: Project = check_sources(&sources)?;
    for language in Language::ALL {
        generate(language, &project)?;
    }
    Ok(project)
}

/// Validates and links the schema files.
pub(crate) fn check_sources(sources: &[SchemaSource]) -> Result<Project, Report> {
    let project: Project = validate_project(sources)?;
    link(&project, sources)
}
