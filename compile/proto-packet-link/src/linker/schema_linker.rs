use crate::{LinkError, Location, Member, NameResolver};
use proto_packet_tree::{
    Import, ModPathRef, Project, SchemaFile, TypeDec, TypeNameRef, WithComments,
};

/// Responsible for linking a schema file.
#[derive(Copy, Clone, Debug)]
pub(crate) struct SchemaLinker<'a> {
    pub(in crate::linker) project: &'a Project,
    pub(in crate::linker) mod_path: ModPathRef<'a>,
    pub(in crate::linker) resolver: NameResolver<'a>,
}

impl<'a> SchemaLinker<'a> {
    //! Construction

    /// Creates a new schema linker for the `schema_file` at the `mod_path` of the `project`.
    pub(crate) fn new(
        project: &'a Project,
        mod_path: ModPathRef<'a>,
        schema_file: &'a SchemaFile,
    ) -> Self {
        Self {
            project,
            mod_path,
            resolver: NameResolver::new(project, mod_path, schema_file),
        }
    }
}

impl SchemaLinker<'_> {
    //! Linking

    /// Links the `schema_file`.
    pub(crate) fn link(self, schema_file: &SchemaFile) -> Result<SchemaFile, LinkError> {
        let mut linked: SchemaFile = SchemaFile::default();
        for import in schema_file.imports() {
            linked
                .add_import(self.link_import(import)?)
                .expect("linking keeps the import names unique");
        }
        for type_dec in schema_file.type_decs() {
            linked
                .add_type_dec(self.link_type_dec(type_dec)?)
                .expect("linking keeps the type names unique");
        }
        Ok(linked)
    }

    /// Links the `import` by checking it is declared in the project.
    fn link_import(self, import: &Import) -> Result<Import, LinkError> {
        match self.project.type_dec_with_name(import.name().to_ref()) {
            Some(_) => Ok(import.clone()),
            None => Err(LinkError::UnresolvedImport {
                mod_path: self.mod_path.into_owned(),
                name: import.name().clone(),
            }),
        }
    }

    /// Links the `type_dec`.
    fn link_type_dec(self, type_dec: &TypeDec) -> Result<TypeDec, LinkError> {
        Ok(match type_dec {
            TypeDec::Struct(structure) => self.link_struct(structure)?.into(),
            TypeDec::Message(message) => self.link_message(message)?.into(),
            TypeDec::Enum(enumeration) => enumeration.clone().into(),
            TypeDec::Variant(variant) => self.link_variant(variant)?.into(),
            TypeDec::Service(service) => self.link_service(service)?.into(),
        })
    }
}

impl SchemaLinker<'_> {
    //! Utils

    /// Gets the location of the `member` of the `type_name` type.
    pub(in crate::linker) fn location(
        self,
        type_name: TypeNameRef<'_>,
        member: Member,
    ) -> Location {
        Location {
            mod_path: self.mod_path.into_owned(),
            type_name: type_name.into_owned(),
            member,
        }
    }
}

/// Copies the comments `from` the element `to` the element.
pub(in crate::linker) fn copy_comments<F, T>(from: &F, to: &mut T)
where
    F: WithComments,
    T: WithComments,
{
    for comment in from.comments() {
        to.add_comment(comment);
    }
}
