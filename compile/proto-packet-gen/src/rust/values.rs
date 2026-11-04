//! The Rust types of field & case values, in each generated type.

use crate::rust::gen_struct_ref::generated_path;
use crate::rust::naming::type_path;
use crate::rust::typing::Typing;
use proto_packet_tree::{SpecialType, TypeTag};

/// The type of a UUID.
const UUID: &str = "proto_packet::types::Uuid";

/// Gets the type of a value of the `tag`, read from a `Ref` type.
pub(crate) fn ref_value_type(typing: Typing<'_>, tag: &TypeTag) -> String {
    match tag {
        TypeTag::Primitive(primitive) => primitive.name().to_string(),
        TypeTag::Special(SpecialType::Uuid) => UUID.to_string(),
        TypeTag::Special(SpecialType::String) => "&'a str".to_string(),
        TypeTag::Named(name) if typing.is_enum(tag) => type_path(name.to_ref()),
        TypeTag::Named(_) => format!("{}<'a>", generated_path(tag, "Ref")),
    }
}

/// Gets the type of a value of the scalar or string `tag`, given to a setter.
pub(crate) fn setter_value_type(tag: &TypeTag) -> String {
    match tag {
        TypeTag::Primitive(primitive) => primitive.name().to_string(),
        TypeTag::Special(SpecialType::Uuid) => UUID.to_string(),
        TypeTag::Special(SpecialType::String) => "&str".to_string(),
        TypeTag::Named(name) => type_path(name.to_ref()),
    }
}

/// Gets the type of a value of the `tag`, read from a `View` trait.
pub(crate) fn view_value_type(typing: Typing<'_>, tag: &TypeTag) -> String {
    match tag {
        TypeTag::Named(_) if !typing.is_enum(tag) => {
            format!("impl {}", generated_path(tag, "View"))
        }
        _ => setter_value_type(tag),
    }
}
