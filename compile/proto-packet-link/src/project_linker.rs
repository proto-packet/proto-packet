use crate::{LinkError, SchemaLinker};
use proto_packet_tree::{Project, SchemaFile};

/// Responsible for linking a whole project.
#[derive(Copy, Clone, Debug, Default)]
#[non_exhaustive]
pub struct ProjectLinker;

impl ProjectLinker {
    //! Link

    /// Links every schema file in the `project`.
    pub fn link(self, project: &Project) -> Result<Project, LinkError> {
        let mut linked: Project = Project::default();
        for (mod_path, schema_file) in project.schema_files() {
            let linker: SchemaLinker = SchemaLinker::from(mod_path.to_ref());
            let linked_schema: SchemaFile = linker.link(schema_file)?;
            linked
                .add_schema_file(mod_path.clone(), linked_schema)
                .expect("duplicate mod path during linking");
        }
        Ok(linked)
    }
}
