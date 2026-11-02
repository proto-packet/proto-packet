use crate::{LinkError, Location, Member, NameResolver};
use proto_packet_tree::{ModPathRef, Project, SchemaFile, TypeDec, TypeNameRef, WithComments};

/// Responsible for linking a schema file.
#[derive(Copy, Clone, Debug)]
pub struct SchemaLinker<'a> {
    pub(in crate::linker) mod_path: ModPathRef<'a>,
    pub(in crate::linker) resolver: NameResolver<'a>,
}

impl<'a> SchemaLinker<'a> {
    //! Construction

    /// Creates a new schema linker for the `schema_file` at the `mod_path` of the `project`.
    pub fn new(
        project: &'a Project,
        mod_path: ModPathRef<'a>,
        schema_file: &'a SchemaFile,
    ) -> Self {
        Self {
            mod_path,
            resolver: NameResolver::new(project, mod_path, schema_file),
        }
    }
}

impl SchemaLinker<'_> {
    //! Linking

    /// Links the `type_dec`.
    pub fn link_type_dec(self, type_dec: &TypeDec) -> Result<TypeDec, LinkError> {
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
