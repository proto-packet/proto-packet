use crate::rust::encoding::IO;
use crate::rust::gen_buf::buf_name;
use crate::rust::gen_struct_ref::{gen_ref_dec, gen_ref_properties, generated_path, ref_name};
use crate::rust::message_encoding::TaggedField;
use crate::rust::naming::{ident, type_path};
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, Var, WithAccess, WithAttributes,
    WithComments, WithFunctions, WithGenerics, WithReceiver, WithResult, WithUnsafeFlag,
    WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet_tree::{PrimitiveType, SpecialType, TypeTag};

/// Generates the `Ref` type of the message with the `schema_name` & the Rust `type_name`.
///
/// It borrows the valid encoding of the message, & finds each field when it is accessed. Its file
/// imports nothing & declares only the `Ref` type, so the prelude names are never shadowed.
pub(crate) fn gen_message_ref(
    typing: Typing<'_>,
    schema_name: &str,
    fields: &[TaggedField<'_>],
    type_name: &str,
) -> String {
    let name: String = ref_name(schema_name);
    let ref_type: RustType = RustType::from(name.as_str()).with_generic(RustType::from("'a"));
    let mut block: Block = Block::default()
        .with_statement(gen_ref_dec(schema_name, &name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_construction(typing, fields, &ref_type))
        .with_statement(EmptyLine::default())
        .with_statement(gen_ref_properties(&ref_type));
    if !fields.is_empty() {
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_fields(typing, fields, &ref_type));
    }
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_decoding(
        typing,
        fields,
        &ref_type,
        type_name,
        &buf_name(schema_name),
    ));
    block.to_code()
}

/// Generates the impl block constructing the `ref_type`.
fn gen_construction(
    typing: Typing<'_>,
    fields: &[TaggedField<'_>],
    ref_type: &RustType,
) -> ImplBlock {
    let signature: Signature = Signature::from("new")
        .with_param(Var::from(("bytes", "&'a [u8]")))
        .with_result(format!("Result<Self, {}::DecodeError>", IO));
    let mut new: Function = Function::from(signature)
        .with_comment(" Creates a new ref of the `bytes`.")
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(" Returns an error if the `bytes` are not a valid encoding.")
        .with_access(Access::Public)
        .with_semi(format!("{}::check_message(bytes)?", IO));
    for tagged in fields {
        new.add_literal(format!(
            "if let Some(value) = unsafe {{ {}::find_field_unchecked(bytes, {}) }} {{ {} }}",
            IO,
            tagged.tag,
            gen_check(typing, *tagged)
        ));
    }
    let new: Function = new.with_literal("Ok(Self { bytes })");

    let signature: Signature = Signature::from("new_unchecked")
        .with_unsafe(true)
        .with_param(Var::from(("bytes", "&'a [u8]")))
        .with_result("Self");
    let new_unchecked: Function = Function::from(signature)
        .with_comment(" Creates a new ref of the `bytes`, without validating them.")
        .with_comment("")
        .with_comment(" # Safety")
        .with_comment(" The `bytes` must be a valid encoding.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true)
        .with_literal("Self { bytes }");

    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Construction")
        .with_function(new)
        .with_function(new_unchecked)
}

/// Generates the statements validating the `value` of the `tagged` field.
fn gen_check(typing: Typing<'_>, tagged: TaggedField<'_>) -> String {
    let tag: &TypeTag = tagged.field.tag;
    match (tag, tagged.size) {
        (TypeTag::Special(SpecialType::String), _) => format!("{}::check_utf8(value)?;", IO),
        (TypeTag::Named(_), Some(size)) if typing.is_enum(tag) => format!(
            "{}::check_len(value, {})?; {}::check_tag_number(unsafe {{ {}::read_u16_unchecked(value, 0) }})?;",
            IO, size, IO, IO
        ),
        (_, Some(size)) => format!("{}::check_len(value, {})?;", IO, size),
        (_, None) => format!("{}::new(value)?;", generated_path(tag, "Ref")),
    }
}

/// Generates the impl block with the getters of the `ref_type`.
fn gen_fields(typing: Typing<'_>, fields: &[TaggedField<'_>], ref_type: &RustType) -> ImplBlock {
    let mut result: ImplBlock = ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Fields");
    for tagged in fields {
        result.add_function(gen_getter(typing, *tagged));
    }
    result
}

/// Generates the getter of the `tagged` field.
fn gen_getter(typing: Typing<'_>, tagged: TaggedField<'_>) -> Function {
    let field = tagged.field;
    let signature: Signature = Signature::from(ident(&field.name))
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("Option<{}>", value_type(typing, field.tag)));
    let mut function: Function = Function::from(signature)
        .with_comment(format!(" Gets the field: `{}`.", field.name))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_semi(format!(
            "let value: &'a [u8] = unsafe {{ {}::find_field_unchecked(self.bytes, {}) }}?",
            IO, tagged.tag
        ))
        .with_literal(format!("Some({})", gen_read(typing, field.tag)));
    if field.is_non_snake_case() {
        function.add_attribute("allow(non_snake_case)");
    }
    function
}

/// Gets the type of a value of the `tag`, read from a `Ref` type.
fn value_type(typing: Typing<'_>, tag: &TypeTag) -> String {
    match tag {
        TypeTag::Primitive(primitive) => primitive.name().to_string(),
        TypeTag::Special(SpecialType::Uuid) => "proto_packet::types::Uuid".to_string(),
        TypeTag::Special(SpecialType::String) => "&'a str".to_string(),
        TypeTag::Named(name) if typing.is_enum(tag) => type_path(name.to_ref()),
        TypeTag::Named(_) => format!("{}<'a>", generated_path(tag, "Ref")),
    }
}

/// Generates the expression reading the `value` bytes of a field of the `tag`.
fn gen_read(typing: Typing<'_>, tag: &TypeTag) -> String {
    match tag {
        TypeTag::Primitive(PrimitiveType::Boolean) => {
            format!("unsafe {{ {}::read_u8_unchecked(value, 0) }} != 0", IO)
        }
        TypeTag::Primitive(primitive) => format!(
            "unsafe {{ {}::read_{}_unchecked(value, 0) }}",
            IO,
            primitive.name()
        ),
        TypeTag::Special(SpecialType::Uuid) => format!(
            "proto_packet::types::Uuid::from_bytes(unsafe {{ {}::read_bytes_unchecked(value, 0) }})",
            IO
        ),
        TypeTag::Special(SpecialType::String) => {
            "unsafe { std::str::from_utf8_unchecked(value) }".to_string()
        }
        TypeTag::Named(name) if typing.is_enum(tag) => format!(
            "{}::from_tag_number(unsafe {{ {}::TagNumber::new_unchecked({}::read_u16_unchecked(value, 0)) }})",
            type_path(name.to_ref()),
            IO,
            IO
        ),
        TypeTag::Named(_) => format!(
            "unsafe {{ {}::new_unchecked(value) }}",
            generated_path(tag, "Ref")
        ),
    }
}

/// Generates the impl block decoding the `ref_type` of the message with the `type_name`.
fn gen_decoding(
    typing: Typing<'_>,
    fields: &[TaggedField<'_>],
    ref_type: &RustType,
    type_name: &str,
    buf_name: &str,
) -> ImplBlock {
    let signature: Signature = Signature::from("decode")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("super::{}", type_name));
    let known: Vec<String> = fields.iter().map(|tagged| tagged.tag.to_string()).collect();
    let mut decoded: String = format!("super::{}::default()", type_name);
    for tagged in fields {
        decoded.push_str(&format!(
            ".{}({})",
            tagged.field.builder_name(),
            gen_decoded_field(typing, *tagged)
        ));
    }
    decoded.push_str(&format!(
        ".with_unknown_fields(unsafe {{ {}::read_unknown_fields_unchecked(self.bytes, &[{}]) }})",
        IO,
        known.join(", ")
    ));
    let decode: Function = Function::from(signature)
        .with_comment(format!(
            " Decodes the [{}](super::{}).",
            type_name, type_name
        ))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(decoded);

    let signature: Signature = Signature::from("to_buf")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("super::{}", buf_name));
    let to_buf: Function = Function::from(signature)
        .with_comment(" Copies the bytes into an owned buf.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ super::{}::new_unchecked(self.bytes.to_vec()) }}",
            buf_name
        ));

    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Decoding")
        .with_function(decode)
        .with_function(to_buf)
}

/// Generates the expression of the owned value of the `tagged` field.
fn gen_decoded_field(typing: Typing<'_>, tagged: TaggedField<'_>) -> String {
    let getter: String = format!("self.{}()", ident(&tagged.field.name));
    match tagged.field.tag {
        TypeTag::Special(SpecialType::String) => format!("{}.map(str::to_string)", getter),
        TypeTag::Named(_) if !typing.is_enum(tagged.field.tag) => {
            format!("{}.map(|value| value.decode())", getter)
        }
        _ => getter,
    }
}
