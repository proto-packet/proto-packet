use crate::layout::{FieldLayout, StructLayout};
use crate::rust::naming::{ident, type_path};
use code_gen::WithStatements;
use code_gen::rust::{
    Access, Function, ImplBlock, Signature, Var, WithAccess, WithAttributes, WithComments,
    WithFunctions, WithResult, WithUnsafeFlag, WithVarParams,
};
use proto_packet_tree::{
    PrimitiveType, Project, SpecialType, Struct, TimeType, TypeDec, TypeTag, WithFieldName,
    WithTypeTag,
};

/// The names of the generated decoding functions of a struct.
pub(crate) const DECODING_FUNCTIONS: [&str; 3] = ["validate", "decode", "decode_unchecked"];

/// The path of the runtime's `io` module.
const IO: &str = "::proto_packet::io";

/// A field of a struct being decoded.
struct Field<'a> {
    name: String,
    tag: &'a TypeTag,
    is_boxed: bool,
    layout: FieldLayout,
}

/// Generates the impl block decoding the `structure` of the `project` with the `layout`.
///
/// The struct is validated, then decoded field by field from its head & data. (see `FORMAT.md`)
pub(crate) fn gen_decoding(
    project: &Project,
    structure: &Struct,
    layout: &StructLayout,
    type_name: &str,
) -> ImplBlock {
    let fields: Vec<Field> = structure
        .fields()
        .iter()
        .zip(layout.fields())
        .map(|(field, layout)| Field {
            name: ident(&field.field_name()),
            tag: field.type_tag(),
            is_boxed: field.is_boxed(),
            layout: *layout,
        })
        .collect();
    ImplBlock::from(type_name)
        .with_comment(" Decoding")
        .with_function(gen_validate(project, &fields, layout))
        .with_function(gen_decode())
        .with_function(gen_decode_unchecked(project, &fields, layout))
}

/// Generates the `validate` function of the struct with the `fields` of the `project`.
///
/// The length is checked first, so the reads are within the bytes.
fn gen_validate(project: &Project, fields: &[Field], layout: &StructLayout) -> Function {
    let signature: Signature = Signature::from("validate")
        .with_param(Var::from(("bytes", "&[u8]")))
        .with_result(format!("::std::result::Result<(), {}::DecodeError>", IO));
    let mut function: Function = Function::from(signature)
        .with_comment(" Validates the encoding of the struct, the `bytes`.")
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(" Returns an error if the `bytes` are not a valid encoding of the struct.")
        .with_access(Access::Public);
    let head: usize = layout.head_size();
    let invalid_length = |expected: &str| {
        format!(
            "return ::std::result::Result::Err({}::DecodeError::InvalidLength {{ expected: {}, \
            actual: bytes.len() }});",
            IO, expected
        )
    };
    let ends: Vec<String> = data_ends(fields);
    if ends.is_empty() {
        let condition: String = if head == 0 {
            "!bytes.is_empty()".to_string()
        } else {
            format!("bytes.len() != {}", head)
        };
        function.add_literal(format!(
            "if {} {{ {} }}",
            condition,
            invalid_length(&head.to_string())
        ));
    } else {
        function.add_literal(format!(
            "if bytes.len() < {} {{ {} }}",
            head,
            invalid_length(&head.to_string())
        ));
        let mut previous: String = format!("{}_usize", head);
        for (end, offset) in ends.iter().zip(length_offsets(fields)) {
            function.add_semi(format!(
                "let {}: usize = {}.saturating_add({}::read_u32(bytes, {}) as usize)",
                end, previous, IO, offset
            ));
            previous = end.clone();
        }
        function.add_literal(format!(
            "if bytes.len() != {} {{ {} }}",
            previous,
            invalid_length(&previous)
        ));
    }
    for (field, location) in fields.iter().zip(locations(fields, layout)) {
        if let Some(statement) = validation(project, field.tag, &location) {
            function.add_semi(statement);
        }
    }
    function.with_literal("::std::result::Result::Ok(())")
}

/// Generates the `decode` function of a struct. (it validates, then decodes unchecked)
fn gen_decode() -> Function {
    let signature: Signature = Signature::from("decode")
        .with_param(Var::from(("bytes", "&[u8]")))
        .with_result(format!("::std::result::Result<Self, {}::DecodeError>", IO));
    Function::from(signature)
        .with_comment(" Decodes the struct from its encoding, the `bytes`.")
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(" Returns an error if the `bytes` are not a valid encoding of the struct.")
        .with_access(Access::Public)
        .with_semi("Self::validate(bytes)?")
        .with_literal("::std::result::Result::Ok(unsafe { Self::decode_unchecked(bytes) })")
}

/// Generates the `decode_unchecked` function of the struct with the `fields` of the `project`.
fn gen_decode_unchecked(project: &Project, fields: &[Field], layout: &StructLayout) -> Function {
    let bytes: &str = if fields.is_empty() { "_bytes" } else { "bytes" };
    let signature: Signature = Signature::from("decode_unchecked")
        .with_unsafe(true)
        .with_param(Var::from((bytes, "&[u8]")))
        .with_result("Self");
    let mut function: Function = Function::from(signature)
        .with_comment(
            " Decodes the struct from its encoding, the `bytes`, without validating them.",
        )
        .with_comment("")
        .with_comment(" # Safety")
        .with_comment(" The `bytes` must be a valid encoding of the struct.")
        .with_attribute("must_use")
        .with_access(Access::Public);
    let mut previous: String = layout.head_size().to_string();
    for (end, offset) in data_ends(fields).iter().zip(length_offsets(fields)) {
        function.add_semi(format!(
            "let {}: usize = {} + unsafe {{ {}::read_u32_unchecked(bytes, {}) }} as usize",
            end, previous, IO, offset
        ));
        previous = end.clone();
    }
    let values: Vec<String> = fields
        .iter()
        .zip(locations(fields, layout))
        .map(|(field, location)| {
            let value: String = value(project, field.tag, &location);
            if field.is_boxed {
                format!("{}: ::std::boxed::Box::new({})", field.name, value)
            } else {
                format!("{}: {}", field.name, value)
            }
        })
        .collect();
    function.with_literal(format!("Self {{ {} }}", values.join(", ")))
}

/// The location of a field's value in the bytes.
enum Location {
    /// A fixed-size value, of the `size`, at the `offset`.
    Fixed { offset: usize, size: usize },

    /// A variable-size value, in the `range` of the data. (ex: `20..end_0`)
    Data { range: String },

    /// A bit of the byte at the `offset`.
    Bit { offset: usize, bit: u8 },
}

/// Gets the locations of the values of the `fields`.
fn locations(fields: &[Field], layout: &StructLayout) -> Vec<Location> {
    let mut start: String = layout.head_size().to_string();
    let mut ends = data_ends(fields).into_iter();
    fields
        .iter()
        .map(|field| match field.layout {
            FieldLayout::Fixed { offset, size } => Location::Fixed { offset, size },
            FieldLayout::Length { .. } => {
                let end: String = ends.next().unwrap_or_default();
                let range: String = format!("{}..{}", start, end);
                start = end;
                Location::Data { range }
            }
            FieldLayout::Bit { offset, bit } => Location::Bit { offset, bit },
        })
        .collect()
}

/// Gets the names of the locals with the end offsets of the data of the variable-size `fields`.
fn data_ends(fields: &[Field]) -> Vec<String> {
    (0..length_offsets(fields).count())
        .map(|index| format!("end_{}", index))
        .collect()
}

/// Gets the offsets of the lengths of the variable-size `fields`, in declaration order.
fn length_offsets<'a>(fields: &'a [Field]) -> impl Iterator<Item = usize> + 'a {
    fields.iter().filter_map(|field| match field.layout {
        FieldLayout::Length { offset } => Some(offset),
        _ => None,
    })
}

/// Gets the optional statement validating the value of the `tag` of the `project` at the
/// `location`. (only values with invalid encodings are validated)
fn validation(project: &Project, tag: &TypeTag, location: &Location) -> Option<String> {
    match (tag, location) {
        (TypeTag::Time(TimeType::Date), Location::Fixed { offset, .. }) => {
            let days: String = format!("{}::read_i32(bytes, {})", IO, offset);
            Some(format!(
                "::proto_packet::types::date_from_days({}).ok_or_else(|| \
                {}::DecodeError::InvalidDate {{ days: {} }})?",
                days, IO, days
            ))
        }
        (TypeTag::Special(SpecialType::String), Location::Data { range }) => Some(format!(
            "::std::str::from_utf8(&bytes[{}]).map_err(|_| {}::DecodeError::InvalidUtf8)?",
            range, IO
        )),
        (TypeTag::Named(name), location) => {
            let path: String = type_path(name.to_ref());
            match (project.type_dec_with_name(name.to_ref()), location) {
                (Some(TypeDec::Enum(_)), Location::Fixed { offset, .. }) => {
                    let tag_number: String = format!("{}::read_u16(bytes, {})", IO, offset);
                    Some(format!(
                        "{}::from_tag_number({}).ok_or_else(|| \
                        {}::DecodeError::InvalidEnumTag {{ tag_number: {} }})?",
                        path, tag_number, IO, tag_number
                    ))
                }
                (_, Location::Fixed { offset, size }) => Some(format!(
                    "{}::validate(&bytes[{}..{}])?",
                    path,
                    offset,
                    offset + size
                )),
                (_, Location::Data { range }) => {
                    Some(format!("{}::validate(&bytes[{}])?", path, range))
                }
                (_, Location::Bit { .. }) => None,
            }
        }
        _ => None,
    }
}

/// Gets the expression reading the valid value of the `tag` of the `project` at the `location`.
fn value(project: &Project, tag: &TypeTag, location: &Location) -> String {
    let read = |name: &str, offset: usize| {
        format!(
            "unsafe {{ {}::read_{}_unchecked(bytes, {}) }}",
            IO, name, offset
        )
    };
    match (tag, location) {
        (_, Location::Bit { offset, bit }) => format!(
            "unsafe {{ {}::read_bit_unchecked(bytes, {}, {}) }}",
            IO, offset, bit
        ),
        (TypeTag::Special(SpecialType::String), Location::Data { range }) => format!(
            "unsafe {{ ::std::str::from_utf8_unchecked(bytes.get_unchecked({})) }}.to_string()",
            range
        ),
        (TypeTag::Named(name), Location::Data { range }) => format!(
            "unsafe {{ {}::decode_unchecked(bytes.get_unchecked({})) }}",
            type_path(name.to_ref()),
            range
        ),
        (_, Location::Data { .. }) => unreachable!("a variable-size value"),
        (tag, Location::Fixed { offset, size }) => {
            let offset: usize = *offset;
            match tag {
                TypeTag::Primitive(PrimitiveType::Float32) => format!(
                    "::proto_packet::types::Float32::new({})",
                    read("f32", offset)
                ),
                TypeTag::Primitive(PrimitiveType::Float64) => format!(
                    "::proto_packet::types::Float64::new({})",
                    read("f64", offset)
                ),
                TypeTag::Primitive(primitive) => read(primitive.name(), offset),
                TypeTag::Special(SpecialType::Uuid) => format!(
                    "::proto_packet::types::Uuid::from_bytes({}.to_le_bytes())",
                    read("u128", offset)
                ),
                TypeTag::Time(TimeType::Timestamp) => read("i64", offset),
                TypeTag::Time(TimeType::Date) => format!(
                    "unsafe {{ ::proto_packet::types::date_from_days(\
                    {}::read_i32_unchecked(bytes, {})).unwrap_unchecked() }}",
                    IO, offset
                ),
                TypeTag::Named(name) => {
                    let path: String = type_path(name.to_ref());
                    match project.type_dec_with_name(name.to_ref()) {
                        Some(TypeDec::Enum(_)) => format!(
                            "unsafe {{ {}::from_tag_number(\
                            {}::read_u16_unchecked(bytes, {})).unwrap_unchecked() }}",
                            path, IO, offset
                        ),
                        _ => format!(
                            "unsafe {{ {}::decode_unchecked(bytes.get_unchecked({}..{})) }}",
                            path,
                            offset,
                            offset + size
                        ),
                    }
                }
                TypeTag::Special(SpecialType::String)
                | TypeTag::Slice { .. }
                | TypeTag::Map { .. } => {
                    unreachable!("a fixed-size value")
                }
            }
        }
    }
}
