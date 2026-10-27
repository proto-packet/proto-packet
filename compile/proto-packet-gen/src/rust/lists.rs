//! The code of the lists, & of the variable-size values containing them. (see `FORMAT.md`)
//!
//! The `depth` of a nested list names its locals & closure variables. (ex: `value_0` & `bytes_0`)
//! The `unsafe` blocks are never nested, since a closure in an `unsafe` block is unsafe too.

use crate::layout::TypeLayout;
use crate::rust::decoding::{IO, fixed_value, struct_path};
use crate::rust::encoding::{checked_length, fixed_statement};
use crate::rust::refs::ref_path;
use crate::rust::typing::Typing;
use code_gen::Expression;
use proto_packet_tree::{PrimitiveType, Project, SpecialType, TimeType, TypeDec, TypeTag};

/// Gets the layout of the elements of the `base` of a list. (lists only have encodable elements)
fn element_layout(project: &Project, base: &TypeTag) -> TypeLayout {
    TypeLayout::new(project, base).expect("an encodable element")
}

/// Checks if the `base` is `u8`, whose lists are their bytes.
fn is_bytes(base: &TypeTag) -> bool {
    matches!(base, TypeTag::Primitive(PrimitiveType::UnsignedInt8))
}

/// Checks if the `tag` is a list of lists read through views. (their view types are complex)
pub(crate) fn has_nested_views(tag: &TypeTag) -> bool {
    match tag {
        TypeTag::Slice { base } => match base.as_ref() {
            TypeTag::Slice { base } => !is_bytes(base),
            _ => false,
        },
        _ => false,
    }
}

/// Gets the expression of the `u32` count of the elements of the list `value`.
fn count(value: &str) -> String {
    format!(
        "u32::try_from({}.len()).expect(\"count out of range\")",
        value
    )
}

/// Gets the expression of the encoding length of the variable-size `value` of the `tag`.
pub(crate) fn variable_len(project: &Project, value: &str, tag: &TypeTag, depth: usize) -> String {
    match tag {
        TypeTag::Special(SpecialType::String) => format!("{}.len()", value),
        TypeTag::Slice { base } => {
            let element: String = format!("value_{}", depth);
            match element_layout(project, base) {
                TypeLayout::Fixed { size: 1, .. } => format!("{}.len()", value),
                TypeLayout::Fixed { size, .. } => format!("{}.len() * {}", value, size),
                TypeLayout::Bit => format!("4 + {}.len().div_ceil(8)", value),
                TypeLayout::Variable => format!(
                    "4 + 4 * {}.len() + {}.iter().map(|{}| {}).sum::<usize>()",
                    value,
                    value,
                    element,
                    variable_len(project, &element, base, depth + 1)
                ),
            }
        }
        _ => format!("{}.encoded_len()", value),
    }
}

/// Gets the statements writing the variable-size `value` of the `tag` to `out`.
pub(crate) fn variable_write(
    project: &Project,
    value: &str,
    tag: &TypeTag,
    depth: usize,
) -> String {
    match tag {
        TypeTag::Special(SpecialType::String) => {
            format!("out.extend_from_slice({}.as_bytes());", value)
        }
        TypeTag::Slice { base } if is_bytes(base) => {
            format!("out.extend_from_slice({}.as_slice());", value)
        }
        TypeTag::Slice { base } => {
            let element: String = format!("value_{}", depth);
            match element_layout(project, base) {
                TypeLayout::Fixed { .. } => format!(
                    "for {} in {}.iter() {{ {} }}",
                    element,
                    value,
                    fixed_statement(project, &element, base, true)
                ),
                TypeLayout::Bit => format!(
                    "out.extend_from_slice(&{}.to_le_bytes()); for chunk in {}.chunks(8) {{ \
                    out.push(chunk.iter().enumerate().fold(0, |byte, (index, bit)| \
                    byte | (u8::from(*bit) << index))); }}",
                    count(value),
                    value
                ),
                TypeLayout::Variable => {
                    let end: String = format!("end_{}", depth);
                    format!(
                        "{{ out.extend_from_slice(&{}.to_le_bytes()); let mut {}: usize = 0; \
                        for {} in {}.iter() {{ {} += {}; out.extend_from_slice(&{}.to_le_bytes()); }} \
                        for {} in {}.iter() {{ {} }} }}",
                        count(value),
                        end,
                        element,
                        value,
                        end,
                        variable_len(project, &element, base, depth + 1),
                        checked_length(&end),
                        element,
                        value,
                        variable_write(project, &element, base, depth + 1)
                    )
                }
            }
        }
        _ => format!("{}.encode_to(out);", value),
    }
}

/// Gets the optional statements validating the variable-size value of the `tag` in the `bytes`.
///
/// Returns `None` if every encoding of the `tag` is valid. (ex: a `[]u8`)
pub(crate) fn variable_validation(
    project: &Project,
    bytes: &str,
    tag: &TypeTag,
    depth: usize,
) -> Option<String> {
    match tag {
        TypeTag::Special(SpecialType::String) => Some(format!(
            "::std::str::from_utf8({}).map_err(|_| {}::DecodeError::InvalidUtf8)?;",
            bytes, IO
        )),
        TypeTag::Slice { base } if is_bytes(base) => None,
        TypeTag::Slice { base } => {
            let element: String = format!("bytes_{}", depth);
            let identity: &str = "::std::convert::identity::<&[u8]>";
            let (list, element_validation): (String, Option<String>) =
                match element_layout(project, base) {
                    TypeLayout::Fixed { size, .. } => (
                        format!(
                            "{}::FixedListRef::new({}, {}, {})?",
                            IO, bytes, size, identity
                        ),
                        fixed_validation(project, &element, base),
                    ),
                    TypeLayout::Bit => (format!("{}::BitListRef::new({})?", IO, bytes), None),
                    TypeLayout::Variable => (
                        format!("{}::VarListRef::new({}, {})?", IO, bytes, identity),
                        variable_validation(project, &element, base, depth + 1),
                    ),
                };
            Some(match element_validation {
                Some(statements) => {
                    format!("for {} in {}.iter() {{ {} }}", element, list, statements)
                }
                None => format!("{};", list),
            })
        }
        _ => Some(format!("{}::validate({})?;", struct_path(tag), bytes)),
    }
}

/// Gets the optional statement validating the fixed-size element of the `tag` in the `bytes`.
fn fixed_validation(project: &Project, bytes: &str, tag: &TypeTag) -> Option<String> {
    match tag {
        TypeTag::Time(TimeType::Date) => {
            let days: String = format!("{}::read_i32({}, 0)", IO, bytes);
            Some(format!(
                "::proto_packet::types::date_from_days({}).ok_or_else(|| \
                {}::DecodeError::InvalidDate {{ days: {} }})?;",
                days, IO, days
            ))
        }
        TypeTag::Named(name) => match project.type_dec_with_name(name.to_ref()) {
            Some(TypeDec::Enum(_)) => {
                let tag_number: String = format!("{}::read_u16({}, 0)", IO, bytes);
                Some(format!(
                    "{}::from_tag_number({}).ok_or_else(|| \
                    {}::DecodeError::InvalidEnumTag {{ tag_number: {} }})?;",
                    struct_path(tag),
                    tag_number,
                    IO,
                    tag_number
                ))
            }
            _ => Some(format!("{}::validate({})?;", struct_path(tag), bytes)),
        },
        _ => None,
    }
}

/// Gets the expression decoding the valid variable-size value of the `tag` in the `bytes`, as its
/// storage type.
pub(crate) fn variable_value(
    project: &Project,
    bytes: &str,
    tag: &TypeTag,
    depth: usize,
) -> String {
    match tag {
        TypeTag::Special(SpecialType::String) => format!(
            "unsafe {{ ::std::str::from_utf8_unchecked({}) }}.to_string()",
            bytes
        ),
        TypeTag::Slice { base } if is_bytes(base) => format!("{}.to_vec()", bytes),
        TypeTag::Slice { base } => {
            let list: String = format!("list_{}", depth);
            let element: String = format!("bytes_{}", depth);
            let read: String = format!("read_{}", depth);
            let (view, element_value): (String, Option<String>) =
                match element_layout(project, base) {
                    TypeLayout::Fixed { size, .. } => (
                        format!("FixedListRef::new_unchecked({}, {}, {})", list, size, read),
                        Some(fixed_value(project, base, 0, &element).unwrap_or_else(|| {
                            format!(
                                "unsafe {{ {}::decode_unchecked({}) }}",
                                struct_path(base),
                                element
                            )
                        })),
                    ),
                    TypeLayout::Bit => (format!("BitListRef::new_unchecked({})", list), None),
                    TypeLayout::Variable => (
                        format!("VarListRef::new_unchecked({}, {})", list, read),
                        Some(variable_value(project, &element, base, depth + 1)),
                    ),
                };
            let read: String = match element_value {
                Some(value) => format!("let {} = |{}: &[u8]| {}; ", read, element, value),
                None => String::default(),
            };
            format!(
                "{{ let {}: &[u8] = {}; {}unsafe {{ {}::{} }}.to_vec() }}",
                list, bytes, read, IO, view
            )
        }
        _ => format!(
            "unsafe {{ {}::decode_unchecked({}) }}",
            struct_path(tag),
            bytes
        ),
    }
}

/// Gets the type & expression reading the valid variable-size value of the `tag` in the `bytes`,
/// a `&'a [u8]`, in place. (ex: a `&'a str`, a `FooRef<'a>` or a list view)
///
/// The elements of a list are read with a closure, inlined, but the elements of a nested list are
/// read with a function pointer, since a closure's type cannot be named in a closure's type.
pub(crate) fn variable_ref(
    typing: Typing<'_>,
    project: &Project,
    bytes: &str,
    tag: &TypeTag,
    depth: usize,
) -> (String, String) {
    match tag {
        TypeTag::Special(SpecialType::String) => (
            "&'a str".to_string(),
            format!("unsafe {{ ::std::str::from_utf8_unchecked({}) }}", bytes),
        ),
        TypeTag::Slice { base } if is_bytes(base) => ("&'a [u8]".to_string(), bytes.to_string()),
        TypeTag::Slice { base } => {
            let element: String = format!("bytes_{}", depth);
            let read: String = format!("read_{}", depth);
            let (element_type, element_value): (String, String) =
                match element_layout(project, base) {
                    TypeLayout::Fixed { .. } => match fixed_value(project, base, 0, &element) {
                        Some(value) if typing.is_converted(base) => (
                            typing.value_type(base).to_code(),
                            format!("{}.value()", value),
                        ),
                        Some(value) => (typing.value_type(base).to_code(), value),
                        None => (
                            format!("{}<'a>", ref_path(base)),
                            format!(
                                "unsafe {{ {}::new_unchecked({}) }}",
                                ref_path(base),
                                element
                            ),
                        ),
                    },
                    TypeLayout::Bit => (String::default(), String::default()),
                    TypeLayout::Variable => {
                        variable_ref(typing, project, &element, base, depth + 1)
                    }
                };
            let reader: String = if depth == 0 {
                format!("impl Fn(&'a [u8]) -> {}", element_type)
            } else {
                format!("fn(&'a [u8]) -> {}", element_type)
            };
            let read: String = if depth == 0 {
                format!("let {} = |{}: &'a [u8]| {}; ", read, element, element_value)
            } else {
                format!(
                    "let {}: {} = |{}: &'a [u8]| {}; ",
                    read, reader, element, element_value
                )
            };
            match element_layout(project, base) {
                TypeLayout::Fixed { size, .. } => (
                    format!("{}::FixedListRef<'a, {}>", IO, reader),
                    format!(
                        "{{ {}unsafe {{ {}::FixedListRef::new_unchecked({}, {}, read_{}) }} }}",
                        read, IO, bytes, size, depth
                    ),
                ),
                TypeLayout::Bit => (
                    format!("{}::BitListRef<'a>", IO),
                    format!("unsafe {{ {}::BitListRef::new_unchecked({}) }}", IO, bytes),
                ),
                TypeLayout::Variable => (
                    format!("{}::VarListRef<'a, {}>", IO, reader),
                    format!(
                        "{{ {}unsafe {{ {}::VarListRef::new_unchecked({}, read_{}) }} }}",
                        read, IO, bytes, depth
                    ),
                ),
            }
        }
        _ => (
            format!("{}<'a>", ref_path(tag)),
            format!("unsafe {{ {}::new_unchecked({}) }}", ref_path(tag), bytes),
        ),
    }
}
