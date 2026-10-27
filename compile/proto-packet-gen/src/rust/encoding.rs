use crate::layout::{FieldLayout, StructLayout};
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

/// Generates the impl block encoding the `structure` of the `project` with the `layout`.
///
/// The struct is encoded field by field, as its head followed by its data. (see `FORMAT.md`)
pub(crate) fn gen_encoding(
    project: &Project,
    structure: &Struct,
    layout: &StructLayout,
    type_name: &str,
) -> ImplBlock {
    let fields: Vec<(String, &TypeTag, FieldLayout)> = structure
        .fields()
        .iter()
        .zip(layout.fields())
        .map(|(field, layout)| (ident(&field.field_name()), field.type_tag(), *layout))
        .collect();
    ImplBlock::from(type_name)
        .with_comment(" Encoding")
        .with_function(gen_encoded_len(&fields, layout))
        .with_function(gen_encode())
        .with_function(gen_encode_to(project, &fields))
}

/// Generates the `encoded_len` method of the struct with the `fields`.
fn gen_encoded_len(fields: &[(String, &TypeTag, FieldLayout)], layout: &StructLayout) -> Function {
    let mut terms: Vec<String> = vec![layout.head_size().to_string()];
    for (name, tag, field_layout) in fields {
        if let FieldLayout::Length { .. } = field_layout {
            terms.push(length_expr(name, tag));
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
fn gen_encode_to(project: &Project, fields: &[(String, &TypeTag, FieldLayout)]) -> Function {
    let mut head: BTreeMap<usize, String> = BTreeMap::default();
    let mut bits: BTreeMap<usize, Vec<String>> = BTreeMap::default();
    let mut data: Vec<String> = Vec::default();
    for (name, tag, layout) in fields {
        match *layout {
            FieldLayout::Fixed { offset, .. } => {
                head.insert(offset, fixed_statement(project, name, tag));
            }
            FieldLayout::Length { offset } => {
                head.insert(
                    offset,
                    format!(
                        "out.extend_from_slice(&u32::try_from({}).expect(\"length out of range\")\
                        .to_le_bytes())",
                        length_expr(name, tag)
                    ),
                );
                data.push(data_statement(name, tag));
            }
            FieldLayout::Bit { offset, bit } => {
                let value: String = format!("u8::from(self.{})", name);
                bits.entry(offset).or_default().push(if bit == 0 {
                    value
                } else {
                    format!("({} << {})", value, bit)
                });
            }
        }
    }
    for (offset, values) in bits {
        head.insert(offset, format!("out.push({})", values.join(" | ")));
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
            " Panics if the encoding of a variable-size field is over `u32::MAX` bytes.",
        );
    }
    for statement in head.into_values().chain(data) {
        function.add_semi(statement);
    }
    function
}

/// Gets the statement writing the fixed-size field with the `name` & `tag` of the `project`.
fn fixed_statement(project: &Project, name: &str, tag: &TypeTag) -> String {
    let bytes = |expr: String| format!("out.extend_from_slice(&{}.to_le_bytes())", expr);
    match tag {
        TypeTag::Primitive(PrimitiveType::Float32 | PrimitiveType::Float64) => {
            bytes(format!("self.{}.value()", name))
        }
        TypeTag::Special(SpecialType::Uuid) => {
            format!("out.extend_from_slice(self.{}.as_bytes())", name)
        }
        TypeTag::Time(TimeType::Date) => bytes(format!(
            "::proto_packet::types::date_to_days(self.{})",
            name
        )),
        TypeTag::Named(type_name) => match project.type_dec_with_name(type_name.to_ref()) {
            Some(TypeDec::Enum(_)) => bytes(format!("self.{}.tag_number()", name)),
            _ => format!("self.{}.encode_to(out)", name),
        },
        _ => bytes(format!("self.{}", name)),
    }
}

/// Gets the expression of the encoding length of the variable-size field with the `name` & `tag`.
fn length_expr(name: &str, tag: &TypeTag) -> String {
    match tag {
        TypeTag::Special(SpecialType::String) => format!("self.{}.len()", name),
        _ => format!("self.{}.encoded_len()", name),
    }
}

/// Gets the statement writing the data of the variable-size field with the `name` & `tag`.
fn data_statement(name: &str, tag: &TypeTag) -> String {
    match tag {
        TypeTag::Special(SpecialType::String) => {
            format!("out.extend_from_slice(self.{}.as_bytes())", name)
        }
        _ => format!("self.{}.encode_to(out)", name),
    }
}
