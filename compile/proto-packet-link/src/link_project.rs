use crate::loops::check_loops;
use crate::{LinkError, SchemaLinker};
use proto_packet_tree::{Project, SchemaFile};

/// Links the `project`.
///
/// Every named type of the linked project is fully qualified & refers to a data type declared in
/// the project, every import refers to a data type declared in the project, no loop of types is
/// without a `box`, & no struct needs itself through its required fields.
pub fn link_project(project: &Project) -> Result<Project, LinkError> {
    let mut linked: Project = Project::default();
    for (mod_path, schema_file) in project.schema_files() {
        let linked_file: SchemaFile =
            SchemaLinker::new(project, mod_path.to_ref(), schema_file).link(schema_file)?;
        linked
            .add_schema_file(mod_path.clone(), linked_file)
            .expect("linking keeps the mod paths unique");
    }
    check_loops(&linked)?;
    Ok(linked)
}
