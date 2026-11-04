use crate::layout::{BitLayout, FieldLayout, StructLayout, ValueLayout};
use crate::rust::encoding::IO;
use crate::rust::fields::Field;
use crate::rust::gen_buf::buf_name;
use crate::rust::gen_forms::{gen_new, gen_new_unchecked, gen_to_buf};
use crate::rust::naming::{ident, is_upper_camel_case, type_path};
use crate::rust::typing::Typing;
use crate::rust::values::ref_value_type;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, Struct as RustStruct, StructField,
    WithAccess, WithAttributes, WithComments, WithDerives, WithFunctions, WithGenerics,
    WithReceiver, WithResult, WithStructFields,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet_tree::{SpecialType, Struct, TypeTag, WithTypeName};

/// Gets the name of the `Ref` type of the struct with the schema `type_name`.
///
/// (ex: `FooRef` for `Foo`, & `SelfRef` for `Self`, whose own name is `Self_`)
pub(crate) fn ref_name(type_name: &str) -> String {
    format!("{}Ref", type_name)
}

/// Gets the path of the generated type with the `suffix`, of the struct of the named `tag`.
///
/// (ex: `crate::a::FooRef` for `a.Foo` & `Ref`)
pub(crate) fn generated_path(tag: &TypeTag, suffix: &str) -> String {
    match tag {
        TypeTag::Named(name) => {
            let path: String = type_path(name.to_ref());
            let mods: &str = path.rsplit_once("::").map_or("", |(mods, _)| mods);
            format!("{}::{}{}", mods, name.to_ref().type_name(), suffix)
        }
        _ => unreachable!("a struct is a named type"),
    }
}

/// Generates the `Ref` type of the `structure` with the Rust `type_name`.
///
/// It borrows the valid encoding of the struct, & reads each field when it is accessed. Its file
/// imports nothing & declares only the `Ref` type, so the prelude names are never shadowed.
pub(crate) fn gen_struct_ref(
    typing: Typing<'_>,
    structure: &Struct,
    fields: &[Field<'_>],
    layout: &StructLayout,
    type_name: &str,
) -> String {
    let name: String = ref_name(&structure.type_name());
    let ref_type: RustType = RustType::from(name.as_str()).with_generic(RustType::from("'a"));
    let mut block: Block = Block::default()
        .with_statement(gen_ref_dec(&structure.type_name(), &name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_construction(typing, fields, layout, &ref_type))
        .with_statement(EmptyLine::default())
        .with_statement(gen_ref_properties(&ref_type));
    if !fields.is_empty() {
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_fields(typing, fields, layout, &ref_type));
    }
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_decoding(
        typing,
        fields,
        &ref_type,
        type_name,
        &buf_name(&structure.type_name()),
    ));
    block.to_code()
}

/// Generates the declaration of the `Ref` type with the `name`.
pub(crate) fn gen_ref_dec(schema_name: &str, name: &str, type_name: &str) -> RustStruct {
    let mut result: RustStruct = RustStruct::from(name)
        .with_comment(format!(
            " A read-only view of an encoded [{}](super::{}).",
            type_name, type_name
        ))
        .with_generic("'a")
        .with_field(StructField::from(("bytes", "&'a [u8]")));
    for derive in ["Copy", "Clone", "Debug"] {
        result.add_derive(derive);
    }
    if !is_upper_camel_case(schema_name) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    result
}

/// Generates the impl block constructing the `ref_type`.
fn gen_construction(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    layout: &StructLayout,
    ref_type: &RustType,
) -> ImplBlock {
    let mut new: Function = gen_new("ref", "&'a [u8]")
        .with_semi(format!("{}::check_len(bytes, {})?", IO, layout.fixed_size));
    if !layout.is_fixed() {
        new.add_semi(format!(
            "{}::check_ends(bytes, {}, {}, {})?",
            IO,
            layout.table_offset,
            layout.entries.len(),
            layout.fixed_size
        ));
    }
    for (field, field_layout) in fields.iter().zip(&layout.fields) {
        if let Some(check) = gen_check(typing, *field, *field_layout, layout) {
            new.add_literal(check);
        }
    }
    let new: Function = new.with_literal("Ok(Self { bytes })");

    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Construction")
        .with_function(new)
        .with_function(gen_new_unchecked("ref", "&'a [u8]"))
}

/// Generates the statement validating the `field`, if its type can be invalid.
fn gen_check(
    typing: Typing<'_>,
    field: Field<'_>,
    field_layout: FieldLayout,
    layout: &StructLayout,
) -> Option<String> {
    let check: String = match (field_layout.value, field.tag) {
        (ValueLayout::Fixed { offset, .. }, _) if typing.is_enum(field.tag) => {
            let check: &str = if field.is_optional {
                "check_optional_tag_number"
            } else {
                "check_tag_number"
            };
            format!(
                "{}::{}(unsafe {{ {} }})?;",
                IO,
                check,
                read_tag_number(offset, "bytes")
            )
        }
        (ValueLayout::Fixed { offset, size }, TypeTag::Named(_)) => format!(
            "{}::new(unsafe {{ {} }})?;",
            generated_path(field.tag, "Ref"),
            read_fixed(offset, size, "bytes")
        ),
        (ValueLayout::Entry { index }, TypeTag::Special(SpecialType::String)) => format!(
            "{}::check_utf8(unsafe {{ {} }})?;",
            IO,
            read_entry("read_entry_unchecked", layout, index, "bytes")
        ),
        (ValueLayout::Entry { index }, _) => format!(
            "{}::new(unsafe {{ {} }})?;",
            generated_path(field.tag, "Ref"),
            read_entry("read_entry_unchecked", layout, index, "bytes")
        ),
        _ => return None,
    };
    Some(match field_layout.presence {
        Some(presence) => format!("if {} {{ {} }}", read_bit(presence, "bytes"), check),
        None => check,
    })
}

/// Generates the impl block with the properties of the `ref_type`.
pub(crate) fn gen_ref_properties(ref_type: &RustType) -> ImplBlock {
    let signature: Signature = Signature::from("as_bytes")
        .with_receiver(Receiver::Borrowed)
        .with_result("&'a [u8]");
    let as_bytes: Function = Function::from(signature)
        .with_comment(" Gets the encoded bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true)
        .with_literal("self.bytes");
    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Properties")
        .with_function(as_bytes)
}

/// Generates the impl block with the getters of the `ref_type`.
fn gen_fields(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    layout: &StructLayout,
    ref_type: &RustType,
) -> ImplBlock {
    let mut result: ImplBlock = ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Fields");
    for (field, field_layout) in fields.iter().zip(&layout.fields) {
        result.add_function(gen_getter(typing, *field, *field_layout, layout));
    }
    result
}

/// Generates the getter of the `field`.
fn gen_getter(
    typing: Typing<'_>,
    field: Field<'_>,
    field_layout: FieldLayout,
    layout: &StructLayout,
) -> Function {
    let value_type: String = ref_value_type(typing, field.tag);
    let result: String = if field.is_optional {
        format!("Option<{}>", value_type)
    } else {
        value_type
    };
    let signature: Signature = Signature::from(ident(&field.name))
        .with_receiver(Receiver::Borrowed)
        .with_result(result);
    let mut function: Function = Function::from(signature)
        .with_comment(format!(" Gets the field: `{}`.", field.name))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(gen_read(typing, field, field_layout, layout));
    if field.is_non_snake_case() {
        function.add_attribute("allow(non_snake_case)");
    }
    function
}

/// Generates the expression reading the `field` of `self.bytes`.
fn gen_read(
    typing: Typing<'_>,
    field: Field<'_>,
    field_layout: FieldLayout,
    layout: &StructLayout,
) -> String {
    const BYTES: &str = "self.bytes";
    if let (TypeTag::Named(name), true, ValueLayout::Fixed { offset, .. }) = (
        field.tag,
        field.is_optional && typing.is_enum(field.tag),
        field_layout.value,
    ) {
        return format!(
            "{}::from_tag_number_optional(u32::from(unsafe {{ {} }}))",
            type_path(name.to_ref()),
            read_tag_number(offset, BYTES)
        );
    }
    let value: String = match (field_layout.value, field.tag) {
        (ValueLayout::Bit(bit), _) => read_bit(bit, BYTES),
        (ValueLayout::Fixed { offset, .. }, TypeTag::Primitive(primitive)) => format!(
            "unsafe {{ {}::read_{}_unchecked({}, {}) }}",
            IO,
            primitive.name(),
            BYTES,
            offset
        ),
        (ValueLayout::Fixed { offset, .. }, TypeTag::Special(SpecialType::Uuid)) => format!(
            "proto_packet::types::Uuid::from_bytes(unsafe {{ {}::read_bytes_unchecked({}, {}) }})",
            IO, BYTES, offset
        ),
        (ValueLayout::Fixed { offset, .. }, TypeTag::Named(name)) if typing.is_enum(field.tag) => {
            format!(
                "{}::from_tag_number(unsafe {{ {}::TagNumber::new_unchecked({}) }})",
                type_path(name.to_ref()),
                IO,
                read_tag_number(offset, BYTES)
            )
        }
        (ValueLayout::Fixed { offset, size }, _) => format!(
            "unsafe {{ {}::new_unchecked({}) }}",
            generated_path(field.tag, "Ref"),
            read_fixed(offset, size, BYTES)
        ),
        (ValueLayout::Entry { index }, TypeTag::Special(SpecialType::String)) => format!(
            "unsafe {{ {} }}",
            read_entry("read_str_unchecked", layout, index, BYTES)
        ),
        (ValueLayout::Entry { index }, _) => format!(
            "unsafe {{ {}::new_unchecked({}) }}",
            generated_path(field.tag, "Ref"),
            read_entry("read_entry_unchecked", layout, index, BYTES)
        ),
    };
    match field_layout.presence {
        Some(presence) => format!(
            "if {} {{ Some({}) }} else {{ None }}",
            read_bit(presence, BYTES),
            value
        ),
        None => value,
    }
}

/// Generates the expression reading the `bit` of the `bytes`.
pub(crate) fn read_bit(bit: BitLayout, bytes: &str) -> String {
    format!(
        "unsafe {{ {}::read_bit_unchecked({}, {}, {}) }}",
        IO, bytes, bit.offset, bit.bit
    )
}

/// Generates the unsafe call reading the enum value at the `offset` of the `bytes`.
fn read_tag_number(offset: usize, bytes: &str) -> String {
    format!("{}::read_u16_unchecked({}, {})", IO, bytes, offset)
}

/// Generates the unsafe expression slicing the fixed struct at the `offset` of the `bytes`.
fn read_fixed(offset: usize, size: usize, bytes: &str) -> String {
    format!("{}.get_unchecked({}..{})", bytes, offset, offset + size)
}

/// Generates the unsafe call to the `read` function for the entry at the `index` of the `bytes`.
pub(crate) fn read_entry(read: &str, layout: &StructLayout, index: usize, bytes: &str) -> String {
    format!(
        "{}::{}({}, {}, {}, {})",
        IO, read, bytes, layout.table_offset, index, layout.fixed_size
    )
}

/// Generates the impl block decoding the `ref_type` of the struct with the `type_name`.
fn gen_decoding(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    ref_type: &RustType,
    type_name: &str,
    buf_name: &str,
) -> ImplBlock {
    let signature: Signature = Signature::from("decode")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("super::{}", type_name));
    let decoded: String = if fields.is_empty() {
        format!("super::{} {{}}", type_name)
    } else {
        let values: Vec<String> = fields
            .iter()
            .map(|field| gen_decoded_field(typing, *field))
            .collect();
        format!("super::{}::new({})", type_name, values.join(", "))
    };
    let decode: Function = Function::from(signature)
        .with_comment(format!(
            " Decodes the [{}](super::{}).",
            type_name, type_name
        ))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(decoded);

    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Decoding")
        .with_function(decode)
        .with_function(gen_to_buf(buf_name))
}

/// Generates the expression of the owned value of the `field`.
fn gen_decoded_field(typing: Typing<'_>, field: Field<'_>) -> String {
    let getter: String = format!("self.{}()", ident(&field.name));
    match field.tag {
        TypeTag::Special(SpecialType::String) if field.is_optional => {
            format!("{}.map(str::to_string)", getter)
        }
        TypeTag::Special(SpecialType::String) => format!("{}.to_string()", getter),
        TypeTag::Named(_) if typing.is_enum(field.tag) => getter,
        TypeTag::Named(_) if field.is_optional => format!("{}.map(|value| value.decode())", getter),
        TypeTag::Named(_) => format!("{}.decode()", getter),
        _ => getter,
    }
}
