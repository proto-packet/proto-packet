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
pub(crate) const DECODING_FUNCTIONS: [&str; 2] = ["decode", "decode_unchecked"];

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
/// The struct is decoded field by field, from its head & data. (see `FORMAT.md`)
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
        .with_function(gen_decode(project, &fields, layout))
        .with_function(gen_decode_unchecked(project, &fields, layout))
}

/// Generates the `decode` function of the struct with the `fields` of the `project`.
///
/// The length is checked first, so the reads are within the bytes.
fn gen_decode(project: &Project, fields: &[Field], layout: &StructLayout) -> Function {
    let signature: Signature = Signature::from("decode")
        .with_param(Var::from(("bytes", "&[u8]")))
        .with_result(format!("::std::result::Result<Self, {}::DecodeError>", IO));
    let mut function: Function = Function::from(signature)
        .with_comment(" Decodes the struct from its encoding, the `bytes`.")
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
        for (end, field) in ends
            .iter()
            .zip(fields.iter().filter(|field| is_length(field)))
        {
            let FieldLayout::Length { offset } = field.layout else {
                unreachable!()
            };
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
    let values: Vec<String> = values(project, fields, layout, false);
    function.add_literal(format!(
        "::std::result::Result::Ok(Self {{ {} }})",
        values.join(", ")
    ));
    function
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
    let lengths = fields.iter().filter(|field| is_length(field));
    for (end, field) in data_ends(fields).iter().zip(lengths) {
        let FieldLayout::Length { offset } = field.layout else {
            unreachable!()
        };
        function.add_semi(format!(
            "let {}: usize = {} + unsafe {{ {}::read_u32_unchecked(bytes, {}) }} as usize",
            end, previous, IO, offset
        ));
        previous = end.clone();
    }
    let values: Vec<String> = values(project, fields, layout, true);
    function.add_literal(format!("Self {{ {} }}", values.join(", ")));
    function
}

/// Gets the names of the locals with the end offsets of the data of the variable-size `fields`.
fn data_ends(fields: &[Field]) -> Vec<String> {
    (0..fields.iter().filter(|field| is_length(field)).count())
        .map(|index| format!("end_{}", index))
        .collect()
}

/// Checks if the `field` is variable-size. (its head item is a length)
fn is_length(field: &Field) -> bool {
    matches!(field.layout, FieldLayout::Length { .. })
}

/// Gets the field initializers of the decoded struct with the `fields` of the `project`.
fn values(
    project: &Project,
    fields: &[Field],
    layout: &StructLayout,
    unchecked: bool,
) -> Vec<String> {
    let mut start: String = layout.head_size().to_string();
    let mut ends = data_ends(fields).into_iter();
    fields
        .iter()
        .map(|field| {
            let value: String = match field.layout {
                FieldLayout::Fixed { offset, size } => {
                    fixed_value(project, field.tag, offset, size, unchecked)
                }
                FieldLayout::Length { .. } => {
                    let end: String = ends.next().unwrap_or_default();
                    let range: String = format!("{}..{}", start, end);
                    start = end;
                    data_value(field.tag, &range, unchecked)
                }
                FieldLayout::Bit { offset, bit } => {
                    read(&format!("bit(bytes, {}, {})", offset, bit), unchecked)
                }
            };
            if field.is_boxed {
                format!("{}: ::std::boxed::Box::new({})", field.name, value)
            } else {
                format!("{}: {}", field.name, value)
            }
        })
        .collect()
}

/// Gets the expression reading the fixed-size value of the `tag` of the `project`, of the `size`
/// at the `offset`.
fn fixed_value(
    project: &Project,
    tag: &TypeTag,
    offset: usize,
    size: usize,
    unchecked: bool,
) -> String {
    let at = |name: &str| read(&format!("{}(bytes, {})", name, offset), unchecked);
    match tag {
        TypeTag::Primitive(PrimitiveType::Float32) => {
            format!("::proto_packet::types::Float32::new({})", at("f32"))
        }
        TypeTag::Primitive(PrimitiveType::Float64) => {
            format!("::proto_packet::types::Float64::new({})", at("f64"))
        }
        TypeTag::Primitive(primitive) => at(primitive.name()),
        TypeTag::Special(SpecialType::Uuid) => format!(
            "::proto_packet::types::Uuid::from_bytes({}.to_le_bytes())",
            at("u128")
        ),
        TypeTag::Time(TimeType::Timestamp) => at("i64"),
        TypeTag::Time(TimeType::Date) => checked(
            "::proto_packet::types::date_from_days",
            &at_raw("i32", offset, unchecked),
            "InvalidDate { days",
            unchecked,
        ),
        TypeTag::Named(name) => {
            let path: String = type_path(name.to_ref());
            match project.type_dec_with_name(name.to_ref()) {
                Some(TypeDec::Enum(_)) => checked(
                    &format!("{}::from_tag_number", path),
                    &at_raw("u16", offset, unchecked),
                    "InvalidEnumTag { tag_number",
                    unchecked,
                ),
                _ => struct_value(&path, &format!("{}..{}", offset, offset + size), unchecked),
            }
        }
        TypeTag::Special(SpecialType::String) | TypeTag::Slice { .. } | TypeTag::Map { .. } => {
            unreachable!("a fixed-size value")
        }
    }
}

/// Gets the expression reading the variable-size value of the `tag` in the `range` of the data.
fn data_value(tag: &TypeTag, range: &str, unchecked: bool) -> String {
    match tag {
        TypeTag::Special(SpecialType::String) if unchecked => format!(
            "unsafe {{ ::std::str::from_utf8_unchecked(bytes.get_unchecked({})) }}.to_string()",
            range
        ),
        TypeTag::Special(SpecialType::String) => format!(
            "::std::str::from_utf8(&bytes[{}]).map_err(|_| {}::DecodeError::InvalidUtf8)?\
            .to_string()",
            range, IO
        ),
        TypeTag::Named(name) => struct_value(&type_path(name.to_ref()), range, unchecked),
        _ => unreachable!("a variable-size value"),
    }
}

/// Gets the expression decoding the struct at the `path` from the `range` of the bytes.
fn struct_value(path: &str, range: &str, unchecked: bool) -> String {
    if unchecked {
        format!(
            "unsafe {{ {}::decode_unchecked(bytes.get_unchecked({})) }}",
            path, range
        )
    } else {
        format!("{}::decode(&bytes[{}])?", path, range)
    }
}

/// Gets the expression of the `read` call of the runtime. (ex: `u32(bytes, 0)` for `read_u32`)
fn read(read: &str, unchecked: bool) -> String {
    if unchecked {
        let (name, args) = read.split_once('(').unwrap_or((read, ")"));
        format!("unsafe {{ {}::read_{}_unchecked({} }}", IO, name, args)
    } else {
        format!("{}::read_{}", IO, read)
    }
}

/// Gets the raw read of the `name`d type at the `offset`, inside an `unsafe` block when unchecked.
fn at_raw(name: &str, offset: usize, unchecked: bool) -> String {
    if unchecked {
        format!("{}::read_{}_unchecked(bytes, {})", IO, name, offset)
    } else {
        format!("{}::read_{}(bytes, {})", IO, name, offset)
    }
}

/// Gets the expression converting the `raw` value with the `convert` function to an `Option`,
/// failing with the `error` (ex: `InvalidDate { days`) of the raw value when it is `None`.
fn checked(convert: &str, raw: &str, error: &str, unchecked: bool) -> String {
    if unchecked {
        format!("unsafe {{ {}({}).unwrap_unchecked() }}", convert, raw)
    } else {
        format!(
            "{}({}).ok_or_else(|| {}::DecodeError::{}: {} }})?",
            convert, raw, IO, error, raw
        )
    }
}
