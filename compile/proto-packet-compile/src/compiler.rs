use crate::CompileError;
use crate::CompileError::{CleanTarget, GenSource, LinkSchema};
use crate::config::{Config, ConfigReader, Language};
use crate::read::ProjectReader;
use crate::write::SourceWriter;
use clerr::Report;
use file_storage::{FolderPath, Operation, Reason};
use proto_packet_gen::Generator;
use proto_packet_gen::rust::RustGenerator;
use proto_packet_link::ProjectLinker;
use proto_packet_tree::Project;

/// Responsible for compiling projects.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Compiler {
    language: Language,
}

impl From<Language> for Compiler {
    fn from(language: Language) -> Self {
        Self { language }
    }
}

impl Compiler {
    //! Compile

    /// Compiles the project in the `source` folder into the `target` folder. The target is
    /// fully wiped (recursively) before generation so stale subdirectories from prior runs
    /// don't linger.
    pub fn compile(&self, source: &FolderPath, target: &FolderPath) -> Result<(), CompileError> {
        let target_path: &std::path::Path = std::path::Path::new(target.as_str());
        if target_path.exists() {
            std::fs::remove_dir_all(target_path).map_err(|_| {
                CleanTarget(file_storage::Error::new(
                    target.clone(),
                    Operation::DeleteFiles,
                    Reason::Other,
                ))
            })?;
        }
        std::fs::create_dir_all(target_path).map_err(|_| {
            CleanTarget(file_storage::Error::new(
                target.clone(),
                Operation::DeleteFiles,
                Reason::Other,
            ))
        })?;

        let config: Config = ConfigReader::default().read_config(source)?;
        let reader: ProjectReader = ProjectReader::from(&config);
        let writer: SourceWriter = SourceWriter::from(target.clone());
        let project: Project = reader.read(source)?;
        let project: Project = ProjectLinker::default()
            .link(&project)
            .map_err(LinkSchema)?;

        let result: Result<(), Report> = match self.language {
            Language::Rust => RustGenerator::default().generate(&project, &writer),
        };

        result.map_err(GenSource)
    }
}
