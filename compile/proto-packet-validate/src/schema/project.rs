use crate::{ValidateError, validate_schema_file_tree};
use proto_packet_parse::SchemaFileTree;
use proto_packet_tree::{ModPath, Project, SchemaFile};

/// Validates the schema file `entries` and assembles them into a project.
///
/// Each entry pairs a pre-validated [ModPath] with the parsed [SchemaFileTree] and its source text.
pub fn validate_project<'a, I>(entries: I) -> Result<Project, ValidateError>
where
    I: IntoIterator<Item = (ModPath, &'a SchemaFileTree, &'a str)>,
{
    let mut project: Project = Project::default();
    for (mod_path, tree, source) in entries {
        let schema_file: SchemaFile = validate_schema_file_tree(tree, source)?;
        project
            .add_schema_file(mod_path, schema_file)
            .map_err(|error| ValidateError::Tree { error })?;
    }
    Ok(project)
}
