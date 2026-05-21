use crate::config::Config;
use crate::{CompileError, SchemaReader};
use file_storage::{FilePath, FolderPath};
use proto_packet_tree::{ModPath, Project, SchemaFile};

/// Responsible for reading projects.
#[derive(Debug)]
pub struct ProjectReader<'a> {
    config: &'a Config,
}

impl<'a> From<&'a Config> for ProjectReader<'a> {
    fn from(config: &'a Config) -> Self {
        Self { config }
    }
}

impl<'a> ProjectReader<'a> {
    //! Read

    /// Reads the project from the `root` folder.
    pub fn read(&self, root: &FolderPath) -> Result<Project, CompileError> {
        let dot_schema: &str = self.config.schema_extension.as_str();
        let separator: char = root.path().file_separator();
        let root_str: &str = root.path().path();

        let files: Vec<FilePath> = root.list_files_as_vec().map_err(CompileError::ReadSchema)?;

        let mut project: Project = Project::default();
        for file in files {
            let file_path: String = file.path().path().to_string();
            if !file_path.ends_with(dot_schema) {
                continue;
            }
            let after_root: &str = file_path
                .strip_prefix(root_str)
                .ok_or_else(|| CompileError::InvalidModPath {
                    root: root.clone(),
                    file_name: file_path.clone(),
                })?
                .trim_start_matches(separator);
            let relative: &str = after_root.strip_suffix(dot_schema).ok_or_else(|| {
                CompileError::InvalidModPath {
                    root: root.clone(),
                    file_name: file_path.clone(),
                }
            })?;
            let mod_path: ModPath =
                ModPath::try_from(relative.replace(separator, ".")).map_err(|_| {
                    CompileError::InvalidModPath {
                        root: root.clone(),
                        file_name: file_path.clone(),
                    }
                })?;
            let display_name: String = after_root.to_string();
            let schema: SchemaFile = SchemaReader::new(file, display_name).read_schema()?;
            project.add_schema_file(mod_path, schema).map_err(|_| {
                CompileError::InvalidModPath {
                    root: root.clone(),
                    file_name: file_path,
                }
            })?;
        }
        Ok(project)
    }
}
