use crate::rust::naming::type_path;
use code_gen::rust::RustType;
use proto_packet_tree::{
    Project, QualifiedName, SpecialType, Struct, TypeDec, TypeTag, WithTypeTag,
};

/// Responsible for the Rust types of schema type tags.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Typing<'a> {
    project: &'a Project,
}

impl<'a> Typing<'a> {
    //! Construction

    /// Creates a new typing for the type tags of the `project`.
    pub(crate) fn new(project: &'a Project) -> Self {
        Self { project }
    }
}

impl Typing<'_> {
    //! Types

    /// Gets the Rust type of the `tag`. (ex: `::std::string::String`)
    pub(crate) fn rust_type(self, tag: &TypeTag) -> RustType {
        match tag {
            TypeTag::Primitive(primitive) => RustType::from(primitive.name()),
            TypeTag::Special(SpecialType::Uuid) => RustType::from("::proto_packet::types::Uuid"),
            TypeTag::Special(SpecialType::String) => RustType::from("::std::string::String"),
            TypeTag::Named(name) => RustType::from(type_path(name.to_ref())),
        }
    }
}

impl Typing<'_> {
    //! Copy

    /// Checks if all the fields of the `structure` are `Copy` types.
    pub(crate) fn is_copy_struct(self, structure: &Struct) -> bool {
        self.is_copy_struct_visiting(structure, &mut Vec::default())
    }

    /// Checks if the `tag` is a `Copy` type, without revisiting the `visiting` structs.
    fn is_copy_visiting(self, tag: &TypeTag, visiting: &mut Vec<QualifiedName>) -> bool {
        match tag {
            TypeTag::Primitive(_) => true,
            TypeTag::Special(special) => *special == SpecialType::Uuid,
            TypeTag::Named(name) => match self.project.type_dec_with_name(name.to_ref()) {
                Some(TypeDec::Enum(_)) => true,
                Some(TypeDec::Struct(structure)) if !visiting.contains(name) => {
                    visiting.push(name.clone());
                    self.is_copy_struct_visiting(structure, visiting)
                }
                _ => false,
            },
        }
    }

    /// Checks if all the fields of the `structure` are `Copy` types.
    fn is_copy_struct_visiting(
        self,
        structure: &Struct,
        visiting: &mut Vec<QualifiedName>,
    ) -> bool {
        structure
            .fields()
            .iter()
            .all(|field| !field.is_boxed() && self.is_copy_visiting(field.type_tag(), visiting))
    }
}

/// Gets the optional type of the `rust_type`.
pub(crate) fn option(rust_type: RustType) -> RustType {
    RustType::from("::std::option::Option").with_generic(rust_type)
}
