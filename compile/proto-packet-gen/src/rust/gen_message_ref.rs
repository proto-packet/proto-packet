use crate::rust::encoding::IO;
use crate::rust::gen_buf::buf_name;
use crate::rust::gen_forms::{gen_new, gen_new_unchecked, gen_to_buf};
use crate::rust::gen_struct_ref::{gen_ref_dec, gen_ref_properties, generated_path, ref_name};
use crate::rust::message_encoding::TaggedField;
use crate::rust::naming::{ident, type_path};
use crate::rust::typing::Typing;
use crate::rust::values::ref_value_type;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, WithAccess, WithAttributes,
    WithComments, WithFunctions, WithGenerics, WithReceiver, WithResult,
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
    let mut new: Function =
        gen_new("ref", "&'a [u8]").with_semi(format!("{}::check_message(bytes)?", IO));
    for tagged in fields {
        new.add_literal(format!(
            "if let Some(value) = unsafe {{ {}::find_field_unchecked(bytes, {}) }} {{ {} }}",
            IO,
            tagged.tag,
            gen_value_check(typing, tagged.field.tag, tagged.size)
        ));
    }
    let new: Function = new.with_literal("Ok(Self { bytes })");

    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Construction")
        .with_function(new)
        .with_function(gen_new_unchecked("ref", "&'a [u8]"))
}

/// Generates the statements validating the `value` bytes of the `tag` with the scalar `size`.
pub(crate) fn gen_value_check(typing: Typing<'_>, tag: &TypeTag, size: Option<usize>) -> String {
    match (tag, size) {
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
        .with_result(format!("Option<{}>", ref_value_type(typing, field.tag)));
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

/// Generates the expression reading the `value` bytes of a field of the `tag`.
pub(crate) fn gen_read(typing: Typing<'_>, tag: &TypeTag) -> String {
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
    let unknown: String = format!(
        "unknown_fields.push({}::UnknownField::new(unsafe {{ {}::TagNumber::new_unchecked(tag) }}, value.to_vec()))",
        IO, IO
    );
    let read: String = if fields.is_empty() {
        format!("{};", unknown)
    } else {
        let arms: Vec<String> = fields
            .iter()
            .map(|tagged| {
                format!(
                    "{} => {{ result.{}({}); }}",
                    tagged.tag,
                    tagged.field.setter_name(),
                    gen_decoded_field(typing, *tagged)
                )
            })
            .collect();
        format!("match tag {{ {} _ => {}, }}", arms.join(" "), unknown)
    };
    let binding: &str = if fields.is_empty() { "" } else { "mut " };
    let decode: Function = Function::from(signature)
        .with_comment(format!(
            " Decodes the [{}](super::{}).",
            type_name, type_name
        ))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_semi(format!(
            "let {}result: super::{} = super::{}::default()",
            binding, type_name, type_name
        ))
        .with_semi(format!(
            "let mut unknown_fields: Vec<{}::UnknownField> = Vec::default()",
            IO
        ))
        .with_literal(format!(
            "for (tag, value) in unsafe {{ {}::message_fields_unchecked(self.bytes) }} {{ {} }}",
            IO, read
        ))
        .with_literal("result.with_unknown_fields(unknown_fields)");

    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Decoding")
        .with_function(decode)
        .with_function(gen_to_buf(buf_name))
}

/// Generates the expression of the owned value of the `tagged` field, from its `value` bytes.
fn gen_decoded_field(typing: Typing<'_>, tagged: TaggedField<'_>) -> String {
    let read: String = gen_read(typing, tagged.field.tag);
    match tagged.field.tag {
        TypeTag::Special(SpecialType::String) => format!("{}.to_string()", read),
        TypeTag::Named(_) if !typing.is_enum(tagged.field.tag) => format!("{}.decode()", read),
        _ => read,
    }
}
