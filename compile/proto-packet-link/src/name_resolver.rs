use crate::LinkError;
use proto_packet_tree::{ModPathRef, QualifiedName, QualifiedNameRef, SchemaFile, WithTypeName};

/// Responsible for resolving names within a single schema file.
///
/// Resolution order:
///   1. Fully qualified names (with a mod path) are returned as-is.
///   2. Local type names are resolved to the current mod path.
///   3. Import effective names are resolved to the import's qualified name.
#[derive(Copy, Clone, Debug)]
pub struct NameResolver<'a> {
    mod_path: ModPathRef<'a>,
    schema_file: &'a SchemaFile,
}

impl<'a> NameResolver<'a> {
    //! Construction

    /// Creates a new name resolver.
    pub fn new(mod_path: ModPathRef<'a>, schema_file: &'a SchemaFile) -> Self {
        Self {
            mod_path,
            schema_file,
        }
    }
}

impl NameResolver<'_> {
    //! Resolve

    /// Resolves the `name` to a fully qualified name.
    ///
    /// Fully qualified names (with a mod path) are returned as-is without verification.
    /// Unqualified names are resolved first against local names, then imports.
    pub fn resolve(self, name: QualifiedNameRef<'_>) -> Result<QualifiedName, LinkError> {
        if name.mod_path().is_some() {
            return Ok(name.into_owned());
        }

        for local_name in self.schema_file.type_decs().iter().map(|t| t.type_name()) {
            if name.type_name() == local_name {
                return Ok(self.mod_path.to_qualified_name(name.type_name()));
            }
        }

        for import in self.schema_file.imports() {
            if name.type_name() == import.effective_name() {
                return Ok(import.name().clone());
            }
        }

        Err(LinkError::UnresolvableName {
            mod_path: self.mod_path.into_owned(),
            type_name: name.type_name().into_owned(),
        })
    }
}
