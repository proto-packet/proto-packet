use crate::{ModPath, QualifiedNameRef, SchemaFile, TreeError, TypeDec};
use std::collections::BTreeMap;

/// A project.
///
/// # Invariants
/// 1. No two schema files can have the same mod path.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Project {
    schema_files: BTreeMap<ModPath, SchemaFile>,
}

impl Project {
    //! Schema Files

    /// Gets the schema files with their mod paths. (sorted by mod path)
    pub fn schema_files(&self) -> impl Iterator<Item = (&ModPath, &SchemaFile)> {
        self.schema_files.iter()
    }

    /// Gets the optional schema file with the `mod_path`.
    pub fn schema_file_with_mod_path<S>(&self, mod_path: S) -> Option<&SchemaFile>
    where
        S: AsRef<str>,
    {
        self.schema_files.get(mod_path.as_ref())
    }

    /// Adds the `schema_file` with the `mod_path`.
    pub fn add_schema_file(
        &mut self,
        mod_path: ModPath,
        schema_file: SchemaFile,
    ) -> Result<(), TreeError> {
        if self.schema_files.contains_key(&mod_path) {
            return Err(TreeError::DuplicateModPath { mod_path });
        }

        self.schema_files.insert(mod_path, schema_file);
        Ok(())
    }

    /// Adds the `schema_file` with the `mod_path`.
    pub fn with_schema_file(
        mut self,
        mod_path: ModPath,
        schema_file: SchemaFile,
    ) -> Result<Self, TreeError> {
        self.add_schema_file(mod_path, schema_file)?;
        Ok(self)
    }
}

impl Project {
    //! Type Declarations

    /// Gets the optional type dec with the qualified `name`.
    ///
    /// Returns `None` if the `name` has no mod path.
    pub fn type_dec_with_name(&self, name: QualifiedNameRef<'_>) -> Option<&TypeDec> {
        self.schema_file_with_mod_path(name.mod_path()?)?
            .type_dec_with_type_name(name.type_name())
    }
}
