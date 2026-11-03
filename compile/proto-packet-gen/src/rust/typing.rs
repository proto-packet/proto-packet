use crate::rust::naming::type_path;
use crate::rust::prelude::{Prelude, PreludeName};
use code_gen::rust::{Reference, RustType};
use proto_packet_tree::{
    Project, QualifiedName, SpecialType, Struct, TypeDec, TypeTag, WithTypeTag,
};

/// Responsible for the Rust types of schema type tags.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Typing<'a> {
    project: &'a Project,
    prelude: Prelude,
}

impl<'a> Typing<'a> {
    //! Construction

    /// Creates a new typing for the type tags of the `project`, in a file with the `prelude`.
    pub(crate) fn new(project: &'a Project, prelude: Prelude) -> Self {
        Self { project, prelude }
    }
}

impl Typing<'_> {
    //! Prelude

    /// Gets the path of the prelude `name` in the file. (ex: `Option`)
    pub(crate) fn prelude(self, name: PreludeName) -> &'static str {
        self.prelude.path(name)
    }
}

impl Typing<'_> {
    //! Types

    /// Gets the Rust type of the `tag`. (ex: `String`)
    pub(crate) fn rust_type(self, tag: &TypeTag) -> RustType {
        match tag {
            TypeTag::Primitive(primitive) => RustType::from(primitive.name()),
            TypeTag::Special(SpecialType::Uuid) => RustType::from("::proto_packet::types::Uuid"),
            TypeTag::Special(SpecialType::String) => {
                RustType::from(self.prelude(PreludeName::String))
            }
            TypeTag::Named(name) => RustType::from(type_path(name.to_ref())),
        }
    }
}

impl Typing<'_> {
    //! Copy

    /// Checks if the `tag` is a `Copy` type.
    pub(crate) fn is_copy(self, tag: &TypeTag) -> bool {
        self.is_copy_visiting(tag, &mut Vec::default())
    }

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
                    let is_copy: bool = self.is_copy_struct_visiting(structure, visiting);
                    visiting.pop();
                    is_copy
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

impl Typing<'_> {
    //! Getters

    /// Gets the return type & expression of the getter for the `tag` field stored in `field`.
    pub(crate) fn getter(
        self,
        tag: &TypeTag,
        field: &str,
        is_optional: bool,
    ) -> (RustType, String) {
        let field: String = format!("self.{}", field);
        let reference = |rust_type: RustType| rust_type.to_ref(Reference::default());
        let (result, expr): (RustType, String) = match tag {
            _ if self.is_copy(tag) => (self.rust_type(tag), field),
            TypeTag::Special(SpecialType::String) => {
                let method: &str = if is_optional { "as_deref" } else { "as_str" };
                (
                    reference(RustType::from("str")),
                    format!("{}.{}()", field, method),
                )
            }
            _ => {
                let expr: String = if is_optional {
                    format!("{}.as_ref()", field)
                } else {
                    format!("&{}", field)
                };
                (reference(self.rust_type(tag)), expr)
            }
        };
        if is_optional {
            (self.option(result), expr)
        } else {
            (result, expr)
        }
    }

    /// Gets the optional type of the `rust_type`.
    pub(crate) fn option(self, rust_type: RustType) -> RustType {
        RustType::from(self.prelude(PreludeName::Option)).with_generic(rust_type)
    }
}
