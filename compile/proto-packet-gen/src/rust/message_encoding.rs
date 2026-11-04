use crate::rust::encoding::{IO, gen_encode};
use crate::rust::fields::Field;
use crate::rust::naming::ident;
use crate::rust::prelude::PreludeName;
use crate::rust::typing::Typing;
use code_gen::WithStatements;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, Signature, Var, WithAccess, WithAttributes,
    WithComments, WithFunctions, WithReceiver, WithResult, WithVarParams,
};
use proto_packet_tree::{PrimitiveType, TypeTag};

/// A message field with its tag number.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TaggedField<'a> {
    pub(crate) field: Field<'a>,
    pub(crate) tag: u16,
    pub(crate) size: Option<usize>,
}

/// Generates the impl block encoding the message with the `type_name` & the `fields` in tag order.
pub(crate) fn gen_message_encoding(
    typing: Typing<'_>,
    type_name: &str,
    fields: &[TaggedField<'_>],
) -> ImplBlock {
    ImplBlock::from(type_name)
        .with_comment(" Encoding")
        .with_function(gen_encoded_len(fields))
        .with_function(gen_encode(typing, "message"))
        .with_function(gen_encode_to(typing, fields))
}

/// Generates the function getting the length of the encoding.
fn gen_encoded_len(fields: &[TaggedField<'_>]) -> Function {
    let signature: Signature = Signature::from("encoded_len")
        .with_receiver(Receiver::Borrowed)
        .with_result("usize");
    let binding: &str = if fields.is_empty() { "" } else { "mut " };
    let mut function: Function = Function::from(signature)
        .with_comment(" Gets the length of the encoding, in bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_semi(format!(
            "let {}count: usize = self.unknown_fields.len()",
            binding
        ))
        .with_semi(format!(
            "let {}values: usize = {}::unknown_fields_len(&self.unknown_fields)",
            binding, IO
        ));
    for tagged in fields {
        let name: String = format!("self.{}", ident(&tagged.field.name));
        function.add_literal(match tagged.size {
            Some(size) => format!(
                "if {}.is_some() {{ count += 1; values += {}; }}",
                name, size
            ),
            None => {
                let len: &str = if tagged.field.tag.is_string() {
                    "len"
                } else {
                    "encoded_len"
                };
                format!(
                    "if let Some(value) = &{} {{ count += 1; values += value.{}(); }}",
                    name, len
                )
            }
        });
    }
    function.with_literal("2 + 6 * count + values")
}

/// Generates the function appending the encoding to the `out` bytes.
fn gen_encode_to(typing: Typing<'_>, fields: &[TaggedField<'_>]) -> Function {
    let param: String = format!("&mut {}<u8>", typing.prelude(PreludeName::Vec));
    let signature: Signature = Signature::from("encode_to")
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from(("out", param.as_str())));
    let known: Vec<String> = fields
        .iter()
        .map(|tagged| format!("usize::from(self.{}.is_some())", ident(&tagged.field.name)))
        .collect();
    let known: String = if known.is_empty() {
        "0".to_string()
    } else {
        known.join(" + ")
    };
    let binding: &str = if fields.is_empty() { "" } else { "mut " };
    let mut function: Function = Function::from(signature)
        .with_comment(" Appends the encoding of the message to the `out` bytes.")
        .with_access(Access::Public)
        .with_semi(format!("let known: usize = {}", known))
        .with_semi(format!(
            "let {}writer: {}::MessageWriter = {}::MessageWriter::new(out, known, &self.unknown_fields)",
            binding, IO, IO
        ));
    for tagged in fields {
        function.add_literal(format!(
            "if let Some(value) = &self.{} {{ {} writer.end_field(); }}",
            ident(&tagged.field.name),
            gen_write(typing, *tagged)
        ));
    }
    function.with_semi("writer.finish()")
}

/// Generates the statement writing the `value` of the `tagged` field.
fn gen_write(typing: Typing<'_>, tagged: TaggedField<'_>) -> String {
    gen_value_write(
        typing,
        tagged.field.tag,
        &format!("writer.field({})", tagged.tag),
    )
}

/// Generates the statement writing the `value` of the `tag` to the `out` expression.
pub(crate) fn gen_value_write(typing: Typing<'_>, tag: &TypeTag, out: &str) -> String {
    match tag {
        TypeTag::Primitive(PrimitiveType::Boolean) => format!("{}.push(u8::from(*value));", out),
        TypeTag::Primitive(_) => format!("{}.extend_from_slice(&value.to_le_bytes());", out),
        TypeTag::Special(_) => format!("{}.extend_from_slice(value.as_bytes());", out),
        TypeTag::Named(_) if typing.is_enum(tag) => format!(
            "{}.extend_from_slice(&value.tag_number().to_le_bytes());",
            out
        ),
        TypeTag::Named(_) => format!("value.encode_to({});", out),
    }
}
