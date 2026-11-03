use crate::Config;
use crate::report::path_report;
use clerr::{Code, Properties, Report};
use proto_packet_parse::SchemaFileTree;
use proto_packet_tree::{ModName, ModPath, Project, SchemaFile};
use std::path::{Path, PathBuf};

/// A schema file of a project, with its source.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SchemaSource {
    /// The mod path. (ex: `a.b` for `a/b.pps`)
    pub mod_path: ModPath,

    /// The file path, relative to the project's source folder. (ex: `a/b.pps`)
    pub path: PathBuf,

    /// The source text.
    pub source: String,
}

/// Reads the schema files of the project in the `source` folder, sorted by path.
///
/// A schema file's mod path is its path from the `source` folder without the extension.
pub fn read_schema_sources(source: &Path, config: &Config) -> Result<Vec<SchemaSource>, Report> {
    let mut paths: Vec<PathBuf> = Vec::default();
    find_schema_files(source, &config.schema_extension, &mut paths)?;
    paths.sort();
    paths
        .into_iter()
        .map(|path| read_schema_source(source, &config.schema_extension, path))
        .collect()
}

/// Finds the schema files in the `folder` & its sub-folders. (symlinks are skipped)
fn find_schema_files(
    folder: &Path,
    extension: &str,
    paths: &mut Vec<PathBuf>,
) -> Result<(), Report> {
    let read_error =
        |error| path_report("C_READ_FOLDER", "could not read the folder", folder, error);
    for entry in std::fs::read_dir(folder).map_err(read_error)? {
        let entry: std::fs::DirEntry = entry.map_err(read_error)?;
        let file_type: std::fs::FileType = entry.file_type().map_err(read_error)?;
        let path: PathBuf = entry.path();
        if file_type.is_dir() {
            find_schema_files(&path, extension, paths)?;
        } else if file_type.is_file() && path.to_string_lossy().ends_with(extension) {
            paths.push(path);
        }
    }
    Ok(())
}

/// Reads the schema file at the `path` in the `source` folder.
fn read_schema_source(
    source: &Path,
    extension: &str,
    path: PathBuf,
) -> Result<SchemaSource, Report> {
    let relative: PathBuf = path.strip_prefix(source).unwrap_or(&path).to_path_buf();
    let mod_path: ModPath = mod_path(&relative, extension)?;
    let text: String = std::fs::read_to_string(&path).map_err(|error| {
        path_report(
            "C_READ_SCHEMA",
            "could not read the schema file",
            &path,
            error,
        )
    })?;
    Ok(SchemaSource {
        mod_path,
        path: relative,
        source: text,
    })
}

/// Gets the mod path of the schema file at the `relative` path.
///
/// Each folder & the file name without the `extension` must be a valid mod name.
fn mod_path(relative: &Path, extension: &str) -> Result<ModPath, Report> {
    let invalid = |message: &str| {
        let properties: Properties = Properties::default()
            .with_property("path", relative.display().to_string())
            .with_property("error", message);
        Report::from(Code::error(
            "C_INVALID_MOD_PATH",
            "invalid schema file path",
        ))
        .with_entry(properties)
    };
    let mut mod_names: Vec<&str> = relative
        .iter()
        .map(|name| name.to_str().ok_or_else(|| invalid("paths must be UTF-8")))
        .collect::<Result<_, _>>()?;
    if let Some(file_name) = mod_names.last_mut() {
        *file_name = file_name.strip_suffix(extension).unwrap_or(file_name);
    }
    for mod_name in &mod_names {
        ModName::validate(mod_name).map_err(|error| invalid(error.message()))?;
    }
    ModPath::new(mod_names.join(".")).map_err(|error| invalid(error.message()))
}

/// Parses & validates the schema `sources` into a project.
pub fn validate_project(sources: &[SchemaSource]) -> Result<Project, Report> {
    let mut project: Project = Project::default();
    for source in sources {
        let file_name: String = source.path.display().to_string();
        let tree: SchemaFileTree = proto_packet_parse::parse(&source.source)
            .map_err(|error| error.report(&file_name, &source.source))?;
        let schema_file: SchemaFile =
            proto_packet_validate::validate_schema_file(&tree, &source.source)
                .map_err(|error| error.report(&file_name, &source.source))?;
        project
            .add_schema_file(source.mod_path.clone(), schema_file)
            .map_err(|error| {
                let properties: Properties =
                    Properties::default().with_property("path", file_name.as_str());
                Report::from(Code::error("C_DUPLICATE_MOD_PATH", error.to_string()))
                    .with_entry(properties)
            })?;
    }
    Ok(project)
}
