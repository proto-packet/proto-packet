use crate::{Import, TreeError, TypeDec, WithTypeName};

/// A schema file.
///
/// # Invariants
/// 1. No two imports can have the same effective name.
/// 2. No two type decs can have the same type name.
/// 3. No import effective name can match a type dec name.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct SchemaFile {
    imports: Vec<Import>,
    type_decs: Vec<TypeDec>,
}

impl SchemaFile {
    //! Imports

    /// Gets the imports.
    pub fn imports(&self) -> &[Import] {
        self.imports.as_slice()
    }

    /// Gets the optional import with the `effective_name`.
    pub fn import_with_effective_name<S>(&self, effective_name: S) -> Option<&Import>
    where
        S: AsRef<str>,
    {
        self.imports
            .iter()
            .find(|import| import.effective_name() == effective_name)
    }

    /// Adds the `import`.
    pub fn add_import(&mut self, import: Import) -> Result<(), TreeError> {
        let effective_name = import.effective_name();
        if self.import_with_effective_name(effective_name).is_some() {
            return Err(TreeError::DuplicateImportName {
                type_name: effective_name.into_owned(),
            });
        }
        if self.type_dec_with_type_name(effective_name).is_some() {
            return Err(TreeError::ImportNameConflictsWithTypeDec {
                type_name: effective_name.into_owned(),
            });
        }

        self.imports.push(import);
        Ok(())
    }

    /// Adds the `import`.
    pub fn with_import(mut self, import: Import) -> Result<Self, TreeError> {
        self.add_import(import)?;
        Ok(self)
    }
}

impl SchemaFile {
    //! Type Declarations

    /// Gets the type decs.
    pub fn type_decs(&self) -> &[TypeDec] {
        self.type_decs.as_slice()
    }

    /// Gets the optional type dec with the `type_name`.
    pub fn type_dec_with_type_name<S>(&self, type_name: S) -> Option<&TypeDec>
    where
        S: AsRef<str>,
    {
        self.type_decs
            .iter()
            .find(|type_dec| type_dec.type_name() == type_name)
    }

    /// Adds the `type_dec`.
    pub fn add_type_dec<D>(&mut self, type_dec: D) -> Result<(), TreeError>
    where
        D: Into<TypeDec>,
    {
        let type_dec: TypeDec = type_dec.into();

        let type_name = type_dec.type_name();
        if self.type_dec_with_type_name(type_name).is_some() {
            return Err(TreeError::DuplicateTypeDecName {
                type_name: type_name.into_owned(),
            });
        }
        if self.import_with_effective_name(type_name).is_some() {
            return Err(TreeError::ImportNameConflictsWithTypeDec {
                type_name: type_name.into_owned(),
            });
        }

        self.type_decs.push(type_dec);
        Ok(())
    }

    /// Adds the `type_dec`.
    pub fn with_type_dec<D>(mut self, type_dec: D) -> Result<Self, TreeError>
    where
        D: Into<TypeDec>,
    {
        self.add_type_dec(type_dec)?;
        Ok(self)
    }
}
