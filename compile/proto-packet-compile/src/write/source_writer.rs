use crate::CompileError;
use clerr::Report;
use code_gen::{Block, CodeBuffer, Statement};
use file_storage::{FilePath, FolderPath, StoragePath};
use proto_packet_gen::Writer;

/// Responsible for writing source files.
#[derive(Debug)]
pub struct SourceWriter {
    root: FolderPath,
}

impl From<FolderPath> for SourceWriter {
    fn from(root: FolderPath) -> Self {
        Self { root }
    }
}

impl Writer for SourceWriter {
    fn write(&self, source: &Block, file_name: &str) -> Result<(), Report> {
        let mut buffer: CodeBuffer = CodeBuffer::default();
        source.write(&mut buffer, 0);
        let source: String = buffer.to_string();

        let file: FilePath = self.file_path(file_name)?;
        file.write_data(source).map_err(CompileError::WriteSource)?;

        Ok(())
    }
}

impl SourceWriter {
    //! File Path

    /// Gets the file path for the `file_name`.
    pub fn file_path(&self, file_name: &str) -> Result<FilePath, CompileError> {
        let path: StoragePath = self
            .root
            .clone_with_extra_capacity(file_name.len())
            .to_path()
            .with_appended(file_name);

        if let Ok(file) = path.to_file() {
            Ok(file)
        } else {
            Err(CompileError::InvalidSourceFileName {
                root: self.root.clone(),
                file_name: file_name.to_string(),
            })
        }
    }
}
