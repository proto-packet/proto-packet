use crate::{LinkError, Location};
use proto_packet_tree::{
    ModPathRef, Project, QualifiedName, QualifiedNameRef, SchemaFile, TypeDec, TypeTag,
};

/// Responsible for resolving the type names of a schema file.
///
/// Resolution order:
/// 1. A name with a mod path is used as is.
/// 2. A local type name resolves to the schema file's mod path.
/// 3. An import's effective name resolves to the import's name.
#[derive(Copy, Clone, Debug)]
pub struct NameResolver<'a> {
    project: &'a Project,
    mod_path: ModPathRef<'a>,
    schema_file: &'a SchemaFile,
}

impl<'a> NameResolver<'a> {
    //! Construction

    /// Creates a new name resolver for the `schema_file` at the `mod_path` of the `project`.
    pub fn new(
        project: &'a Project,
        mod_path: ModPathRef<'a>,
        schema_file: &'a SchemaFile,
    ) -> Self {
        Self {
            project,
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

impl NameResolver<'_> {
    //! Linking

    /// Links the type `tag` at the `location` by resolving its named type.
    ///
    /// The named type must be declared in the project & be a data type, not a service.
    pub fn link_type_tag(self, tag: &TypeTag, location: &Location) -> Result<TypeTag, LinkError> {
        match tag {
            TypeTag::Named(name) => self.link_name(name, location).map(TypeTag::Named),
            TypeTag::Slice { base } => Ok(self.link_type_tag(base, location)?.into_slice()),
            TypeTag::Map { .. } => todo!(),
            TypeTag::Primitive(_) | TypeTag::Special(_) | TypeTag::Time(_) => Ok(tag.clone()),
        }
    }

    /// Links the type `name` at the `location`.
    fn link_name(
        self,
        name: &QualifiedName,
        location: &Location,
    ) -> Result<QualifiedName, LinkError> {
        let unresolved = || LinkError::UnresolvedType {
            location: location.clone(),
            name: name.clone(),
        };
        let resolved: QualifiedName = self.resolve(name.to_ref()).ok_or_else(unresolved)?;
        match self.project.type_dec_with_name(resolved.to_ref()) {
            None => Err(unresolved()),
            Some(TypeDec::Service(_)) => Err(LinkError::ServiceAsType {
                location: location.clone(),
                name: resolved,
            }),
            Some(_) => Ok(resolved),
        }
    }
}
