use crate::layout::{FieldLayout, StructLayout, UnitLayout, ValueLayout};
use crate::rust::fields::Field;
use crate::rust::naming::ident;
use crate::rust::prelude::PreludeName;
use crate::rust::typing::Typing;
use code_gen::WithStatements;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, Signature, Var, WithAccess, WithAttributes,
    WithComments, WithFunctions, WithReceiver, WithResult, WithVarParams,
};
use proto_packet_tree::{SpecialType, TypeTag};

/// The path of the runtime's `io` module.
pub(crate) const IO: &str = "proto_packet::io";

/// Generates the impl block encoding the struct with the `type_name`, the `fields` & the `layout`.
pub(crate) fn gen_encoding(
    typing: Typing<'_>,
    type_name: &str,
    fields: &[Field<'_>],
    layout: &StructLayout,
) -> ImplBlock {
    ImplBlock::from(type_name)
        .with_comment(" Encoding")
        .with_function(gen_encoded_len(layout))
        .with_function(gen_encode(typing))
        .with_function(gen_encode_to(typing, fields, layout))
}

/// Generates the function getting the length of the encoding.
fn gen_encoded_len(layout: &StructLayout) -> Function {
    let signature: Signature = Signature::from("encoded_len")
        .with_receiver(Receiver::Borrowed)
        .with_result("usize");
    Function::from(signature)
        .with_comment(" Gets the length of the encoding, in bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(layout.fixed_size.to_string())
}

/// Generates the function encoding the struct.
fn gen_encode(typing: Typing<'_>) -> Function {
    let bytes: String = bytes_type(typing);
    let signature: Signature = Signature::from("encode")
        .with_receiver(Receiver::Borrowed)
        .with_result(bytes.as_str());
    Function::from(signature)
        .with_comment(" Encodes the struct.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_semi(format!(
            "let mut result: {} = {}::with_capacity(self.encoded_len())",
            bytes,
            typing.prelude(PreludeName::Vec)
        ))
        .with_semi("self.encode_to(&mut result)")
        .with_literal("result")
}

/// Generates the function appending the encoding to the `out` bytes.
fn gen_encode_to(typing: Typing<'_>, fields: &[Field<'_>], layout: &StructLayout) -> Function {
    let statements: Vec<String> = layout
        .units
        .iter()
        .flat_map(|unit| gen_unit(typing, fields, layout, *unit))
        .collect();
    let out: &str = if statements.is_empty() { "_out" } else { "out" };
    let signature: Signature = Signature::from("encode_to")
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from((out, format!("&mut {}", bytes_type(typing)))));
    let mut function: Function = Function::from(signature)
        .with_comment(" Appends the encoding of the struct to the `out` bytes.")
        .with_access(Access::Public);
    for statement in statements {
        function.add_literal(statement);
    }
    function
}

/// Generates the statements appending the `unit` of the fixed block.
fn gen_unit(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    layout: &StructLayout,
    unit: UnitLayout,
) -> Vec<String> {
    match unit {
        UnitLayout::Field { index } => {
            vec![gen_fixed_field(typing, fields[index], layout.fields[index])]
        }
        UnitLayout::EndTable => unreachable!("a fixed struct has no end table"),
        UnitLayout::Bits { offset, size } => (offset..offset + size)
            .map(|offset| gen_bits(fields, layout, offset))
            .collect(),
        UnitLayout::Padding { size } => vec![format!("{}::write_zeros(out, {});", IO, size)],
    }
}

/// Generates the statement appending the fixed-size `field`.
///
/// An absent field keeps its place as zeros.
fn gen_fixed_field(typing: Typing<'_>, field: Field<'_>, layout: FieldLayout) -> String {
    let name: String = format!("self.{}", ident(&field.name));
    let value: String = if field.is_optional {
        format!("{}.unwrap_or_default()", name)
    } else {
        name.clone()
    };
    match field.tag {
        TypeTag::Special(SpecialType::Uuid) => {
            format!("out.extend_from_slice({}.as_bytes());", value)
        }
        TypeTag::Primitive(_) => format!("out.extend_from_slice(&{}.to_le_bytes());", value),
        _ if typing.is_enum(field.tag) => {
            let tag_number: String = if field.is_optional {
                format!("{}.as_ref().map_or(0, |value| value.tag_number())", name)
            } else {
                format!("{}.tag_number()", name)
            };
            format!("out.extend_from_slice(&{}.to_le_bytes());", tag_number)
        }
        _ if field.is_optional => {
            let ValueLayout::Fixed { size, .. } = layout.value else {
                unreachable!("a fixed-size field is in the fixed block")
            };
            format!(
                "match &{} {{ Some(value) => value.encode_to(out), None => {}::write_zeros(out, {}), }}",
                name, IO, size
            )
        }
        _ => format!("{}.encode_to(out);", name),
    }
}

/// Generates the statement appending the byte of the bit group at the `offset`.
fn gen_bits(fields: &[Field<'_>], layout: &StructLayout, offset: usize) -> String {
    let mut bits: Vec<String> = Vec::default();
    for (field, field_layout) in fields.iter().zip(&layout.fields) {
        let name: String = format!("self.{}", ident(&field.name));
        if let Some(presence) = field_layout.presence
            && presence.offset == offset
        {
            bits.push(gen_bit(&format!("{}.is_some()", name), presence.bit));
        }
        if let ValueLayout::Bit(bit) = field_layout.value
            && bit.offset == offset
        {
            let value: String = if field.is_optional {
                format!("{}.unwrap_or_default()", name)
            } else {
                name
            };
            bits.push(gen_bit(&value, bit.bit));
        }
    }
    format!("out.push({});", bits.join(" | "))
}

/// Generates the expression of the `bool` `value` at the `bit` of a byte.
fn gen_bit(value: &str, bit: u8) -> String {
    if bit == 0 {
        format!("u8::from({})", value)
    } else {
        format!("(u8::from({}) << {})", value, bit)
    }
}

/// Gets the type of the encoded bytes.
fn bytes_type(typing: Typing<'_>) -> String {
    format!("{}<u8>", typing.prelude(PreludeName::Vec))
}
