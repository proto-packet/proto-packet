use crate::{ModPath, SchemaFile, TreeError};
use std::collections::HashMap;

/// A project.
///
/// # Invariants
/// 1. No two schema files can have the same mod path.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Project {
    schemas: HashMap<ModPath, SchemaFile>,
}

impl Project {
    //! Schema Files

    /// Gets the schema files keyed by their mod path.
    pub fn schema_files(&self) -> impl Iterator<Item = (&ModPath, &SchemaFile)> {
        self.schemas.iter()
    }

    /// Gets the optional schema file with the `mod_path`.
    pub fn schema_file_with_mod_path<S>(&self, mod_path: S) -> Option<&SchemaFile>
    where
        S: AsRef<str>,
    {
        self.schemas.get(mod_path.as_ref())
    }

    /// Adds the `schema_file` with the `mod_path`.
    pub fn add_schema_file<P, F>(&mut self, mod_path: P, schema_file: F) -> Result<(), TreeError>
    where
        P: Into<ModPath>,
        F: Into<SchemaFile>,
    {
        let mod_path: ModPath = mod_path.into();
        let schema_file: SchemaFile = schema_file.into();

        if self.schemas.contains_key(&mod_path) {
            return Err(TreeError::DuplicateModPathName { mod_path });
        }

        self.schemas.insert(mod_path, schema_file);
        Ok(())
    }

    /// Adds the `schema_file` with the `mod_path`.
    pub fn with_schema_file<P, F>(mut self, mod_path: P, schema_file: F) -> Result<Self, TreeError>
    where
        P: Into<ModPath>,
        F: Into<SchemaFile>,
    {
        self.add_schema_file(mod_path, schema_file)?;
        Ok(self)
    }
}
