use crate::rust::naming::type_path;
use code_gen::rust::{Reference, RustType};
use proto_packet_tree::{
    PrimitiveType, Project, QualifiedName, SpecialType, Struct, TimeType, TypeDec, TypeTag,
    WithTypeTag,
};

/// Responsible for the Rust types of schema type tags.
///
/// A type tag has a value type, used by the generated API, & a storage type, used by the generated
/// fields. They only differ for floats, which are stored with total ordering & hashing.
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

    /// Gets the value type of the `tag`. (ex: `f32`)
    pub(crate) fn value_type(self, tag: &TypeTag) -> RustType {
        match tag {
            TypeTag::Primitive(primitive) => RustType::from(primitive.name()),
            TypeTag::Special(SpecialType::Uuid) => RustType::from("::proto_packet::types::Uuid"),
            TypeTag::Special(SpecialType::String) => RustType::from("::std::string::String"),
            TypeTag::Time(TimeType::Timestamp) => RustType::from("i64"),
            TypeTag::Time(TimeType::Date) => RustType::from("::proto_packet::types::Date"),
            TypeTag::Named(name) => RustType::from(type_path(name.to_ref())),
            TypeTag::Slice { .. } => self.storage_type(tag),
        }
    }

    /// Gets the storage type of the `tag`. (ex: `::proto_packet::types::Float32`)
    pub(crate) fn storage_type(self, tag: &TypeTag) -> RustType {
        match tag {
            TypeTag::Primitive(PrimitiveType::Float32) => {
                RustType::from("::proto_packet::types::Float32")
            }
            TypeTag::Primitive(PrimitiveType::Float64) => {
                RustType::from("::proto_packet::types::Float64")
            }
            TypeTag::Slice { base } => {
                RustType::from("::std::vec::Vec").with_generic(self.storage_type(base))
            }
            _ => self.value_type(tag),
        }
    }
}

impl Typing<'_> {
    //! Conversions

    /// Checks if the value & storage types of the `tag` differ.
    pub(crate) fn is_converted(self, tag: &TypeTag) -> bool {
        matches!(
            tag,
            TypeTag::Primitive(PrimitiveType::Float32 | PrimitiveType::Float64)
        )
    }

    /// Converts the value `expr` of the `tag` to its storage type.
    pub(crate) fn storage_expr(self, tag: &TypeTag, expr: &str) -> String {
        if self.is_converted(tag) {
            format!("{}.into()", expr)
        } else {
            expr.to_string()
        }
    }

    /// Converts the storage `expr` of the `tag` to its value type.
    pub(crate) fn value_expr(self, tag: &TypeTag, expr: &str) -> String {
        self.storage_expr(tag, expr)
    }

    /// Converts the optional value `expr` of the `tag` to its optional storage type.
    pub(crate) fn storage_expr_option(self, tag: &TypeTag, expr: &str) -> String {
        if self.is_converted(tag) {
            format!("{}.map(::std::convert::Into::into)", expr)
        } else {
            expr.to_string()
        }
    }

    /// Converts the optional storage `expr` of the `tag` to its optional value type.
    pub(crate) fn value_expr_option(self, tag: &TypeTag, expr: &str) -> String {
        self.storage_expr_option(tag, expr)
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
            TypeTag::Primitive(_) | TypeTag::Time(_) => true,
            TypeTag::Special(special) => *special == SpecialType::Uuid,
            TypeTag::Slice { .. } => false,
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
            _ if self.is_copy(tag) => {
                let expr: String = if is_optional {
                    self.value_expr_option(tag, &field)
                } else {
                    self.value_expr(tag, &field)
                };
                (self.value_type(tag), expr)
            }
            TypeTag::Special(SpecialType::String) => {
                let method: &str = if is_optional { "as_deref" } else { "as_str" };
                (
                    reference(RustType::from("str")),
                    format!("{}.{}()", field, method),
                )
            }
            TypeTag::Slice { base } => {
                let method: &str = if is_optional { "as_deref" } else { "as_slice" };
                let slice: RustType = self.storage_type(base).to_slice();
                (reference(slice), format!("{}.{}()", field, method))
            }
            _ => {
                let expr: String = if is_optional {
                    format!("{}.as_ref()", field)
                } else {
                    format!("&{}", field)
                };
                (reference(self.value_type(tag)), expr)
            }
        };
        if is_optional {
            (option(result), expr)
        } else {
            (result, expr)
        }
    }
}

/// Gets the optional type of the `rust_type`.
pub(crate) fn option(rust_type: RustType) -> RustType {
    RustType::from("::std::option::Option").with_generic(rust_type)
}
