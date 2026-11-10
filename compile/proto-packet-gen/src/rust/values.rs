//! The Rust types of field & case values, in each generated type.

use crate::rust::gen_struct_ref::generated_path;
use crate::rust::naming::type_path;
use crate::rust::typing::Typing;
use proto_packet_tree::{PrimitiveType, SpecialType, TypeTag};

/// The type of a UUID.
const UUID: &str = "proto_packet::types::Uuid";

/// The type of the bytes of a `[]u8`.
pub(crate) const BYTES: &str = "proto_packet::types::Bytes";

/// The type of a list read from its bytes.
pub(crate) const LIST_REF: &str = "proto_packet::io::ListRef";

/// The type of a map read from its bytes.
pub(crate) const MAP_REF: &str = "proto_packet::io::MapRef";

/// The trait of any form of a list.
const LIST_VIEW: &str = "proto_packet::io::ListView";

/// The trait of any form of a map.
const MAP_VIEW: &str = "proto_packet::io::MapView";

/// Gets the Rust type of the `primitive`.
///
/// # Note
/// - A float is the runtime's type with one NaN, so the generated types have `Eq`, `Ord` & `Hash`.
/// - A time is the runtime's type for it, so its unit can't be mixed up.
/// - A decimal is the runtime's type with its scale, so two scales can't be mixed up.
pub(crate) fn primitive_type(primitive: PrimitiveType) -> String {
    let runtime: &str = match primitive {
        PrimitiveType::Float32 => "Float32",
        PrimitiveType::Float64 => "Float64",
        PrimitiveType::Timestamp => "Timestamp",
        PrimitiveType::Microtime => "Microtime",
        PrimitiveType::Nanotime => "Nanotime",
        PrimitiveType::Date => "Date",
        PrimitiveType::Duration => "Duration",
        PrimitiveType::Decimal { scale } => {
            return format!("proto_packet::types::Decimal<{}>", scale);
        }
        _ => return primitive.name().to_string(),
    };
    format!("proto_packet::types::{}", runtime)
}

/// Gets the integer type of the bits of the `primitive`, if it's a float. (ex: `u32` for `f32`)
pub(crate) const fn float_bits(primitive: PrimitiveType) -> Option<&'static str> {
    match primitive {
        PrimitiveType::Float32 => Some("u32"),
        PrimitiveType::Float64 => Some("u64"),
        _ => None,
    }
}

/// Gets the type of a value of the `tag`, read from a `Ref` type.
pub(crate) fn ref_value_type(typing: Typing<'_>, tag: &TypeTag) -> String {
    match tag {
        TypeTag::Primitive(primitive) => primitive_type(*primitive),
        TypeTag::Special(SpecialType::Uuid) => UUID.to_string(),
        TypeTag::Special(SpecialType::String) => "&'a str".to_string(),
        TypeTag::Named(name) if typing.is_enum(tag) => type_path(name.to_ref()),
        TypeTag::Named(_) => format!("{}<'a>", generated_path(tag, "Ref")),
        TypeTag::List(_) if tag.is_bytes() => "&'a [u8]".to_string(),
        TypeTag::List(element) => format!("{}<'a, {}>", LIST_REF, typing.rust_type(element)),
        TypeTag::Map { key, value } => format!(
            "{}<'a, {}, {}>",
            MAP_REF,
            typing.rust_type(key),
            typing.rust_type(value)
        ),
    }
}

/// Gets the type of a value of the scalar or string `tag`, given to a setter.
pub(crate) fn setter_value_type(tag: &TypeTag) -> String {
    match tag {
        TypeTag::Primitive(primitive) => primitive_type(*primitive),
        TypeTag::Special(SpecialType::Uuid) => UUID.to_string(),
        TypeTag::Special(SpecialType::String) => "&str".to_string(),
        TypeTag::Named(name) => type_path(name.to_ref()),
        TypeTag::List(_) | TypeTag::Map { .. } => {
            unreachable!("a list or a map has no setter in place")
        }
    }
}

/// Gets the type of a value of the `tag`, read from a `View` trait.
///
/// # Note
/// - A nested type, a list or a map is an `impl` of its view trait, since each form has its own
///   type for it.
pub(crate) fn view_value_type(typing: Typing<'_>, tag: &TypeTag) -> String {
    match tag {
        TypeTag::Named(_) if !typing.is_enum(tag) => {
            format!("impl {}", generated_path(tag, "View"))
        }
        TypeTag::List(_) if tag.is_bytes() => "&[u8]".to_string(),
        TypeTag::List(element) => format!(
            "impl {}<Item = {}>",
            LIST_VIEW,
            view_value_type(typing, element)
        ),
        TypeTag::Map { key, value } => format!(
            "impl {}<{}, Key = {}, Value = {}>",
            MAP_VIEW,
            typing.rust_type(key),
            view_value_type(typing, key),
            view_value_type(typing, value)
        ),
        _ => setter_value_type(tag),
    }
}
