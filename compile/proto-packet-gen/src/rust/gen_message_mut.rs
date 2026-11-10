use crate::rust::encoding::IO;
use crate::rust::gen_struct_mut::{
    gen_mut_construction, gen_mut_dec, gen_mut_properties, gen_setter_signature, is_struct,
    mut_name,
};
use crate::rust::gen_struct_ref::{generated_path, ref_name};
use crate::rust::message_encoding::TaggedField;
use crate::rust::typing::Typing;
use crate::rust::values::setter_value_type;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, WithAccess, WithAttributes,
    WithComments, WithFunctions, WithReceiver, WithResult,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet_tree::{PrimitiveType, TypeTag};

/// Generates the `Mut` type of the message with the `schema_name` & the Rust `type_name`.
///
/// It edits the valid encoding of the message in place, & every edit keeps it valid. Any setter can
/// fail, since adding a field takes room. Its file imports nothing & declares only the `Mut` type,
/// so the prelude names are never shadowed.
pub(crate) fn gen_message_mut(
    typing: Typing<'_>,
    schema_name: &str,
    fields: &[TaggedField<'_>],
    type_name: &str,
) -> String {
    let name: String = mut_name(schema_name);
    let ref_name: String = ref_name(schema_name);
    let any_lifetime: RustType = RustType::from(name.as_str()).with_generic(RustType::from("'_"));
    let mut block: Block = Block::default()
        .with_statement(gen_mut_dec(schema_name, &name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_mut_construction(&name, &ref_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_mut_properties(&any_lifetime, &ref_name));
    if fields.iter().any(|tagged| !tagged.field.tag.is_list()) {
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_fields(typing, fields, &any_lifetime));
    }
    block.to_code()
}

/// Generates the impl block with the method of each of the `fields`.
fn gen_fields(typing: Typing<'_>, fields: &[TaggedField<'_>], mut_type: &RustType) -> ImplBlock {
    let mut result: ImplBlock = ImplBlock::from(mut_type.clone()).with_comment(" Fields");
    for tagged in fields {
        if tagged.field.tag.is_list() {
            continue;
        }
        let mut function: Function = if is_struct(typing, tagged.field) {
            gen_message_child(*tagged, "self.bytes")
        } else {
            gen_setter(typing, *tagged)
        };
        if tagged.field.is_non_snake_case() {
            function.add_attribute("allow(non_snake_case)");
        }
        result.add_function(function);
    }
    result
}

/// Generates the accessor of the nested `tagged` field, to edit it in place.
///
/// The `bytes` expression is the type's bytes, as a mutable slice.
fn gen_message_child(tagged: TaggedField<'_>, bytes: &str) -> Function {
    let child: String = generated_path(tagged.field.tag, "Mut");
    let signature: Signature = Signature::from(tagged.field.accessor_name())
        .with_receiver(Receiver::BorrowedMut)
        .with_result(format!("Option<{}<'_>>", child));
    Function::from(signature)
        .with_comment(format!(
            " Gets the mut of the field: `{}`.",
            tagged.field.name
        ))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_semi(format!(
            "let value: &mut [u8] = unsafe {{ {}::find_field_mut_unchecked({}, {}) }}?",
            IO, bytes, tagged.tag
        ))
        .with_literal(format!(
            "Some(unsafe {{ {}::new_unchecked(value) }})",
            child
        ))
}

/// Generates the impl block of the `Buf` type with the `name`, with an accessor for each nested
/// struct or message of the `fields`.
///
/// A `Buf` type can't lend these through a temporary `Mut` type, so it has its own.
pub(crate) fn gen_message_buf_accessors(
    typing: Typing<'_>,
    fields: &[TaggedField<'_>],
    name: &str,
) -> Option<ImplBlock> {
    if !fields.iter().any(|tagged| is_struct(typing, tagged.field)) {
        return None;
    }
    let mut result: ImplBlock = ImplBlock::from(name).with_comment(" Fields");
    for tagged in fields {
        if is_struct(typing, tagged.field) {
            let mut function: Function = gen_message_child(*tagged, "self.bytes.as_mut_slice()");
            if tagged.field.is_non_snake_case() {
                function.add_attribute("allow(non_snake_case)");
            }
            result.add_function(function);
        }
    }
    Some(result)
}

/// Generates the setter of the `tagged` field.
fn gen_setter(typing: Typing<'_>, tagged: TaggedField<'_>) -> Function {
    let field = tagged.field;
    let mut function: Function = Function::from(gen_setter_signature(field, true))
        .with_comment(format!(" Sets the field: `{}`.", field.name))
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(
            " Returns an error, leaving the message unchanged, if the message would change its",
        )
        .with_comment(" length.")
        .with_access(Access::Public)
        .with_semi(format!(
            "let value: Option<{}> = value.into()",
            setter_value_type(field.tag)
        ));
    let replace = |value: &str| -> String {
        format!(
            "{}::replace_field(self.bytes, {}, {})",
            IO, tagged.tag, value
        )
    };
    if field.tag.is_string() {
        return function.with_literal(replace("value.map(str::as_bytes)"));
    }
    let size: usize = tagged.size.expect("a scalar has a size");
    function.add_semi(format!(
        "let bytes: Option<[u8; {}]> = value.map(|value| {})",
        size,
        gen_bytes(typing, field.tag)
    ));
    function.with_literal(replace("bytes.as_ref().map(|bytes| bytes.as_slice())"))
}

/// Generates the expression of the bytes of a scalar `value` of the `tag`.
pub(crate) fn gen_bytes(typing: Typing<'_>, tag: &TypeTag) -> &'static str {
    match tag {
        TypeTag::Primitive(PrimitiveType::Boolean) => "[u8::from(value)]",
        TypeTag::Primitive(_) => "value.to_le_bytes()",
        TypeTag::Special(_) => "*value.as_bytes()",
        TypeTag::Named(_) if typing.is_enum(tag) => "value.tag_number().to_le_bytes()",
        TypeTag::Named(_) => unreachable!("a nested type has an accessor"),
        TypeTag::List(_) => unreachable!("a list has no setter in place"),
    }
}
