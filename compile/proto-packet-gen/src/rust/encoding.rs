use crate::layout::{BitLayout, FieldLayout, StructLayout, ValueLayout};
use crate::rust::lists::{variable_len, variable_write};
use crate::rust::naming::ident;
use code_gen::WithStatements;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, Signature, Var, WithAccess, WithAttributes,
    WithComments, WithFunctions, WithReceiver, WithResult, WithVarParams,
};
use proto_packet_tree::{
    PrimitiveType, Project, SpecialType, Struct, TimeType, TypeDec, TypeTag, WithFieldName,
    WithTypeTag,
};
use std::collections::BTreeMap;

/// The names of the generated encoding methods of a struct.
pub(crate) const ENCODING_METHODS: [&str; 3] = ["encoded_len", "encode", "encode_to"];

/// A field of a struct being encoded.
struct Field<'a> {
    name: String,
    tag: &'a TypeTag,
    is_optional: bool,
    layout: FieldLayout,
}

/// Generates the impl block encoding the `structure` of the `project` with the `layout`.
///
/// The struct is encoded field by field, as its head followed by its data. (see `FORMAT.md`)
pub(crate) fn gen_encoding(
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
            is_optional: field.is_optional(),
            layout: *layout,
        })
        .collect();
    ImplBlock::from(type_name)
        .with_comment(" Encoding")
        .with_function(gen_encoded_len(project, &fields, layout))
        .with_function(gen_encode())
        .with_function(gen_encode_to(project, &fields))
}

/// Generates the `encoded_len` method of the struct with the `fields`.
///
/// An absent variable-size value has no data.
fn gen_encoded_len(project: &Project, fields: &[Field], layout: &StructLayout) -> Function {
    let mut terms: Vec<String> = vec![layout.head_size().to_string()];
    for field in fields {
        if let ValueLayout::Length { .. } = field.layout.value {
            terms.push(if field.is_optional {
                format!(
                    "self.{}.as_ref().map_or(0, |value| {})",
                    field.name,
                    variable_len(project, "value", field.tag, 0)
                )
            } else {
                variable_len(project, &format!("self.{}", field.name), field.tag, 0)
            });
        }
    }
    let signature: Signature = Signature::from("encoded_len")
        .with_receiver(Receiver::Borrowed)
        .with_result("usize");
    Function::from(signature)
        .with_comment(" Gets the length of the encoding, in bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(terms.join(" + "))
}

/// Generates the `encode` method of a struct.
fn gen_encode() -> Function {
    let signature: Signature = Signature::from("encode")
        .with_receiver(Receiver::Borrowed)
        .with_result("::std::vec::Vec<u8>");
    Function::from(signature)
        .with_comment(" Encodes the struct.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_semi(
            "let mut result: ::std::vec::Vec<u8> = \
            ::std::vec::Vec::with_capacity(self.encoded_len())",
        )
        .with_semi("self.encode_to(&mut result)")
        .with_literal("result")
}

/// Generates the `encode_to` method of the struct with the `fields` of the `project`.
///
/// The head items are written in offset order, with the bits of each byte combined.
fn gen_encode_to(project: &Project, fields: &[Field]) -> Function {
    let mut head: BTreeMap<usize, String> = BTreeMap::default();
    let mut bits: BTreeMap<usize, Vec<String>> = BTreeMap::default();
    let mut data: Vec<String> = Vec::default();
    let mut add_bit = |bit: BitLayout, value: String| {
        let value: String = format!("u8::from({})", value);
        bits.entry(bit.offset).or_default().push(if bit.bit == 0 {
            value
        } else {
            format!("({} << {})", value, bit.bit)
        });
    };
    for field in fields {
        let name: &str = &field.name;
        if let Some(presence) = field.layout.presence {
            add_bit(presence, format!("self.{}.is_some()", name));
        }
        match field.layout.value {
            ValueLayout::Fixed { offset, size } => {
                let statement: String = if !field.is_optional {
                    fixed_statement(project, &format!("self.{}", name), field.tag, false)
                } else if field.layout.presence.is_some() {
                    format!(
                        "match &self.{} {{ ::std::option::Option::Some(value) => {{ {} }} \
                        ::std::option::Option::None => {{ out.extend_from_slice(&[0; {}]); }} }}",
                        name,
                        fixed_statement(project, "value", field.tag, true),
                        size
                    )
                } else {
                    format!(
                        "out.extend_from_slice(&self.{}.as_ref().map_or(0, |value| \
                        value.tag_number()).to_le_bytes());",
                        name
                    )
                };
                head.insert(offset, statement);
            }
            ValueLayout::Length { offset } => {
                let length: String = if field.is_optional {
                    format!(
                        "self.{}.as_ref().map_or(u32::MAX, |value| {})",
                        name,
                        checked_length(&variable_len(project, "value", field.tag, 0))
                    )
                } else {
                    checked_length(&variable_len(
                        project,
                        &format!("self.{}", name),
                        field.tag,
                        0,
                    ))
                };
                head.insert(
                    offset,
                    format!("out.extend_from_slice(&{}.to_le_bytes());", length),
                );
                data.push(if field.is_optional {
                    format!(
                        "if let ::std::option::Option::Some(value) = &self.{} {{ {} }}",
                        name,
                        variable_write(project, "value", field.tag, 0)
                    )
                } else {
                    variable_write(project, &format!("self.{}", name), field.tag, 0)
                });
            }
            ValueLayout::Bit(bit) => {
                let value: String = if field.is_optional {
                    format!("self.{} == ::std::option::Option::Some(true)", name)
                } else {
                    format!("self.{}", name)
                };
                add_bit(bit, value);
            }
        }
    }
    for (offset, values) in bits {
        head.insert(offset, format!("out.push({});", values.join(" | ")));
    }

    let out: &str = if fields.is_empty() { "_out" } else { "out" };
    let signature: Signature = Signature::from("encode_to")
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from((out, "&mut ::std::vec::Vec<u8>")));
    let mut function: Function = Function::from(signature)
        .with_comment(" Appends the encoding of the struct to the `out` bytes.")
        .with_access(Access::Public);
    if !data.is_empty() {
        function.add_comment("");
        function.add_comment(" # Panics");
        function.add_comment(
            " Panics if the encoding of a variable-size field is `u32::MAX` bytes or more.",
        );
    }
    for statement in head.into_values().chain(data) {
        function.add_literal(statement);
    }
    function
}

/// Gets the statement writing the fixed-size `value` of the `tag` of the `project`.
///
/// The `value` is a reference when `is_ref`. (ex: `value` of `Some(value)`)
pub(crate) fn fixed_statement(
    project: &Project,
    value: &str,
    tag: &TypeTag,
    is_ref: bool,
) -> String {
    let bytes = |expr: String| format!("out.extend_from_slice(&{}.to_le_bytes());", expr);
    match tag {
        TypeTag::Primitive(PrimitiveType::Float32 | PrimitiveType::Float64) => {
            bytes(format!("{}.value()", value))
        }
        TypeTag::Special(SpecialType::Uuid) => {
            format!("out.extend_from_slice({}.as_bytes());", value)
        }
        TypeTag::Time(TimeType::Date) => bytes(format!(
            "::proto_packet::types::date_to_days({}{})",
            if is_ref { "*" } else { "" },
            value
        )),
        TypeTag::Named(type_name) => match project.type_dec_with_name(type_name.to_ref()) {
            Some(TypeDec::Enum(_)) => bytes(format!("{}.tag_number()", value)),
            _ => format!("{}.encode_to(out);", value),
        },
        _ => bytes(value.to_string()),
    }
}

/// Gets the `u32` of the `length`, panicking if it is `u32::MAX` or more. (the absent length)
pub(crate) fn checked_length(length: &str) -> String {
    format!(
        "u32::try_from({}).ok().filter(|length| *length != u32::MAX).expect(\"length out of range\")",
        length
    )
}
