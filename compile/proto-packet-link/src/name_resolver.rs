use proto_packet_tree::{ModPathRef, QualifiedName, QualifiedNameRef, SchemaFile};

/// Responsible for resolving the type names of a schema file.
///
/// Resolution order:
/// 1. A name with a mod path is used as is.
/// 2. A local type name resolves to the schema file's mod path.
/// 3. An import's effective name resolves to the import's name.
#[derive(Copy, Clone, Debug)]
pub struct NameResolver<'a> {
    mod_path: ModPathRef<'a>,
    schema_file: &'a SchemaFile,
}

impl<'a> NameResolver<'a> {
    //! Construction

    /// Creates a new name resolver for the `schema_file` at the `mod_path`.
    pub fn new(mod_path: ModPathRef<'a>, schema_file: &'a SchemaFile) -> Self {
        Self {
            mod_path,
            schema_file,
        }
    }
}

impl NameResolver<'_> {
    //! Resolution

    /// Resolves the `name` to a fully qualified name.
    ///
    /// Returns `None` if the `name` has no mod path & is neither a local type nor an import. The
    /// resolved name is not checked against the project.
    pub fn resolve(self, name: QualifiedNameRef<'_>) -> Option<QualifiedName> {
        if name.mod_path().is_some() {
            Some(name.into_owned())
        } else if self
            .schema_file
            .type_dec_with_type_name(name.type_name())
            .is_some()
        {
            Some(QualifiedName::from_parts(self.mod_path, name.type_name()))
        } else {
            self.schema_file
                .import_with_effective_name(name.type_name())
                .map(|import| import.name().clone())
        }
    }
}
