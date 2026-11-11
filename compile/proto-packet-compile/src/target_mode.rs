use crate::report::path_report;
use clerr::{Code, Properties, Report};
use proto_packet_gen::GeneratedFile;
use serde::Deserialize;
use std::fs::{DirEntry, File, FileType};
use std::io::Read;
use std::path::{Path, PathBuf};

/// What a compile does with the files already in its target folder.
#[must_use]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetMode {
    /// Replaces the files, which must all be generated files.
    #[default]
    Replace,
}

impl TargetMode {
    //! Constants

    /// The most bytes read from a file to get its first line.
    const FIRST_LINE_LIMIT: u64 = 256;
}

impl TargetMode {
    //! Write

    /// Writes the generated `files` to the `target` folder.
    ///
    /// # Note
    /// - The folder itself is kept, so a target of `.` or a symlink to a folder works.
    ///
    /// # Errors
    /// - The folder holds a file that was not generated. Nothing is deleted or written.
    pub(crate) fn write(self, target: &Path, files: &[GeneratedFile]) -> Result<(), Report> {
        match self {
            Self::Replace => {
                if target.exists() {
                    self.check_generated(target)?;
                    self.clear_folder(target)
                        .map_err(|error| self.write_error(target, error))?;
                }
            }
        }
        for file in files {
            let path: PathBuf = target.join(&file.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| self.write_error(parent, error))?;
            }
            std::fs::write(&path, &file.code).map_err(|error| self.write_error(&path, error))?;
        }
        Ok(())
    }

    /// Checks the `folder` holds only generated files, so that clearing it deletes no other file.
    ///
    /// # Note
    /// - The entries are checked in the order of their names, so the same file is always reported.
    /// - A symlink is not a generated file.
    fn check_generated(self, folder: &Path) -> Result<(), Report> {
        let read_error = |path: &Path, error: std::io::Error| {
            path_report("C_READ_TARGET", "could not read the target", path, error)
        };
        let mut entries: Vec<DirEntry> = std::fs::read_dir(folder)
            .and_then(Iterator::collect)
            .map_err(|error| read_error(folder, error))?;
        entries.sort_by_key(DirEntry::file_name);
        for entry in entries {
            let path: PathBuf = entry.path();
            let file_type: FileType = entry
                .file_type()
                .map_err(|error| read_error(&path, error))?;
            if file_type.is_dir() {
                self.check_generated(&path)?;
            } else if !file_type.is_file()
                || !self
                    .is_generated(&path)
                    .map_err(|error| read_error(&path, error))?
            {
                let properties: Properties =
                    Properties::default().with_property("path", path.display().to_string());
                return Err(Report::from(Code::error(
                    "C_TARGET_NOT_GENERATED",
                    "the target folder holds a file that was not generated",
                ))
                .with_entry(properties));
            }
        }
        Ok(())
    }

    /// Checks if the first line of the file at the `path` has the marker of a generated file.
    fn is_generated(self, path: &Path) -> std::io::Result<bool> {
        let mut start: Vec<u8> = Vec::default();
        File::open(path)?
            .take(Self::FIRST_LINE_LIMIT)
            .read_to_end(&mut start)?;
        let line: &[u8] = start
            .split(|byte| *byte == b'\n')
            .next()
            .unwrap_or_default();
        let marker: &[u8] = GeneratedFile::MARKER.as_bytes();
        Ok(line.windows(marker.len()).any(|window| window == marker))
    }

    /// Deletes the contents of the `folder`. (symlinks are deleted, not followed)
    fn clear_folder(self, folder: &Path) -> std::io::Result<()> {
        for entry in std::fs::read_dir(folder)? {
            let entry: DirEntry = entry?;
            if entry.file_type()?.is_dir() {
                std::fs::remove_dir_all(entry.path())?;
            } else {
                std::fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }

    /// Creates the report for the `error` of writing the `path`.
    fn write_error(self, path: &Path, error: std::io::Error) -> Report {
        path_report("C_WRITE_TARGET", "could not write the target", path, error)
    }
}
