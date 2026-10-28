//! The code of the maps. (see `FORMAT.md`)
//!
//! A map's keys & values are encoded as lists, so their code is the code of the lists of the keys &
//! of the values. The `depth` of a nested map names its locals. (ex: `keys_0` & `values_0`)

use crate::rust::decoding::IO;
use crate::rust::encoding::checked_length;
use crate::rust::lists::{
    element_reader, elements_len, elements_write, variable_ref, variable_validation, view,
};
use crate::rust::typing::Typing;
use proto_packet_tree::{PrimitiveType, Project, SpecialType, TypeTag};

/// The function reading an element as its bytes.
const IDENTITY: &str = "::std::convert::identity::<&[u8]>";

/// Gets the list of the `base` elements. (ex: the keys of a map as a `[]K`)
fn list(base: &TypeTag) -> TypeTag {
    base.clone().into_slice()
}

/// Gets the expression of the encoding length of the `map` of the `key` & `value` types.
pub(crate) fn map_len(
    project: &Project,
    map: &str,
    key: &TypeTag,
    value: &TypeTag,
    depth: usize,
) -> String {
    let len: String = format!("{}.len()", map);
    format!(
        "4 + {} + {}",
        elements_len(project, &len, &format!("{}.keys()", map), key, depth),
        elements_len(project, &len, &format!("{}.values()", map), value, depth)
    )
}

/// Gets the statements writing the `map` of the `key` & `value` types to `out`.
///
/// A `BTreeMap` iterates its keys in increasing order, so the keys are written in order.
pub(crate) fn map_write(
    project: &Project,
    map: &str,
    key: &TypeTag,
    value: &TypeTag,
    depth: usize,
) -> String {
    let len: String = format!("{}.len()", map);
    let keys: String = format!("{}.keys()", map);
    let values: String = format!("{}.values()", map);
    format!(
        "out.extend_from_slice(&{}.to_le_bytes()); {} {}",
        checked_length(&elements_len(project, &len, &keys, key, depth)),
        elements_write(project, &len, &keys, key, depth),
        elements_write(project, &len, &values, value, depth)
    )
}

/// Gets the statements validating the map of the `key` & `value` types in the `bytes`.
///
/// The keys & values are validated as lists, then the counts & the uniqueness of the keys.
pub(crate) fn map_validation(
    project: &Project,
    bytes: &str,
    key: &TypeTag,
    value: &TypeTag,
    depth: usize,
) -> String {
    let keys: String = format!("keys_{}", depth);
    let values: String = format!("values_{}", depth);
    let read: String = format!("read_{}", depth);
    let mut statements: Vec<String> = vec![format!(
        "let ({}, {}): (&[u8], &[u8]) = {}::split_map({})?;",
        keys, values, IO, bytes
    )];
    statements.extend(variable_validation(project, &keys, &list(key), depth));
    statements.extend(variable_validation(project, &values, &list(value), depth));
    let read: &str = if is_compared_as_bytes(key) {
        IDENTITY
    } else {
        statements.push(element_reader(project, &read, key, depth));
        &read
    };
    statements.push(format!(
        "let {} = unsafe {{ {} }};",
        keys,
        view(project, &keys, key, read)
    ));
    statements.push(format!(
        "let {} = unsafe {{ {} }};",
        values,
        view(project, &values, value, IDENTITY)
    ));
    statements.push(format!("{}::MapRef::new({}, {})?;", IO, keys, values));
    format!("{{ {} }}", statements.join(" "))
}

/// Checks if the keys of the `key` type are equal when their bytes are, so their uniqueness is
/// checked without decoding them. (ex: a `string`, but not a struct with an absent field)
fn is_compared_as_bytes(key: &TypeTag) -> bool {
    match key {
        TypeTag::Special(SpecialType::String) => true,
        TypeTag::Slice { base } => matches!(
            base.as_ref(),
            TypeTag::Primitive(PrimitiveType::UnsignedInt8)
        ),
        _ => false,
    }
}

/// Gets the expression decoding the valid map of the `key` & `value` types in the `bytes`, as a
/// `BTreeMap`.
pub(crate) fn map_value(
    project: &Project,
    bytes: &str,
    key: &TypeTag,
    value: &TypeTag,
    depth: usize,
) -> String {
    let keys: String = format!("keys_{}", depth);
    let values: String = format!("values_{}", depth);
    let read_keys: String = format!("read_keys_{}", depth);
    let read_values: String = format!("read_values_{}", depth);
    format!(
        "{{ let ({}, {}): (&[u8], &[u8]) = unsafe {{ {}::split_map_unchecked({}) }}; {}{}\
        let {} = unsafe {{ {} }}; let {} = unsafe {{ {} }}; \
        {}.iter().zip({}.iter()).collect::<::std::collections::BTreeMap<_, _>>() }}",
        keys,
        values,
        IO,
        bytes,
        element_reader(project, &read_keys, key, depth),
        element_reader(project, &read_values, value, depth),
        keys,
        view(project, &keys, key, &read_keys),
        values,
        view(project, &values, value, &read_values),
        keys,
        values
    )
}

/// Gets the type & expression reading the valid map of the `key` & `value` types in the `bytes`,
/// a `&'a [u8]`, in place, as a `MapRef` of the views of its keys & values.
pub(crate) fn map_ref(
    typing: Typing<'_>,
    project: &Project,
    bytes: &str,
    key: &TypeTag,
    value: &TypeTag,
    depth: usize,
) -> (String, String) {
    let keys: String = format!("keys_{}", depth);
    let values: String = format!("values_{}", depth);
    let (keys_type, keys_expr): (String, String) =
        variable_ref(typing, project, &keys, &list(key), depth);
    let (values_type, values_expr): (String, String) =
        variable_ref(typing, project, &values, &list(value), depth);
    (
        format!("{}::MapRef<{}, {}>", IO, keys_type, values_type),
        format!(
            "{{ let ({}, {}): (&'a [u8], &'a [u8]) = unsafe {{ {}::split_map_unchecked({}) }}; \
            let {} = {}; let {} = {}; unsafe {{ {}::MapRef::new_unchecked({}, {}) }} }}",
            keys, values, IO, bytes, keys, keys_expr, values, values_expr, IO, keys, values
        ),
    )
}
