use crate::{Config, SchemaSource, link, read_schema_sources, validate_project};
use clerr::Report;
use proto_packet_tree::Project;
use std::path::Path;

/// Parses, validates and links the project in the source folder.
///
/// # Note
/// - Nothing is written.
/// - Generating code has its own errors, which only compiling finds.
pub fn check(source: &Path) -> Result<Project, Report> {
    let config: Config = Config::read(source)?;
    let sources: Vec<SchemaSource> = read_schema_sources(source, &config)?;
    check_sources(&sources)
}

/// Validates and links the schema files.
pub(crate) fn check_sources(sources: &[SchemaSource]) -> Result<Project, Report> {
    let project: Project = validate_project(sources)?;
    link(&project, sources)
}
