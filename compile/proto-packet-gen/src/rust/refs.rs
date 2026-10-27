use crate::layout::{BitLayout, StructLayout, ValueLayout};
use crate::rust::bufs::buf_name;
use crate::rust::decoding::{IO, bit_value, data_length, fixed_value, optional_enum_value};
use crate::rust::naming::{ident, is_snake_case, is_upper_camel_case, type_path};
use crate::rust::typing::{Typing, option};
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, Struct as RustStruct, StructField,
    Var, WithAccess, WithAttributes, WithComments, WithDerives, WithFunctions, WithGenerics,
    WithReceiver, WithResult, WithStructFields, WithUnsafeFlag, WithVarParams,
};
use code_gen::{Block, EmptyLine, WithStatements};
use proto_packet_tree::{
    Project, SpecialType, Struct, TypeTag, WithFieldName, WithTypeName, WithTypeTag,
};

/// The names of the generated functions of a `Ref` type, other than its field getters.
pub(crate) const REF_FUNCTIONS: [&str; 5] =
    ["new", "new_unchecked", "as_bytes", "decode", "to_buf"];

/// Gets the Rust name of the `Ref` type of the struct with the schema `type_name`.
///
/// (ex: `FooRef` for `Foo`, & `SelfRef` for `Self`, whose own name is `Self_`)
pub(crate) fn ref_name(type_name: &str) -> String {
    ident(&format!("{}Ref", type_name))
}

/// Gets the Rust path of the `Ref` type of the struct of the named `tag`.
///
/// (ex: `crate::a::FooRef` for `a.Foo`)
fn ref_path(tag: &TypeTag) -> String {
    match tag {
        TypeTag::Named(name) => {
            let path: String = type_path(name.to_ref());
            let mods: &str = path.rsplit_once("::").map_or("", |(mods, _)| mods);
            format!("{}::{}", mods, ref_name(&name.to_ref().type_name()))
        }
        _ => unreachable!("a named type"),
    }
}

/// Generates the `Ref` type of the `structure` of the `project` with the `layout`.
///
/// It borrows the valid encoding of the struct, & reads each field from its offset when the field
/// is accessed. (see `FORMAT.md`)
pub(crate) fn gen_ref(
    typing: Typing<'_>,
    project: &Project,
    structure: &Struct,
    layout: &StructLayout,
    type_name: &str,
) -> Block {
    let name: String = ref_name(&structure.type_name());
    let ref_type: RustType = RustType::from(name.as_str()).with_generic(RustType::from("'a"));
    let mut block: Block = Block::default()
        .with_statement(gen_dec(structure, &name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_construction(&ref_type, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_encoding(
            &ref_type,
            &buf_name(&structure.type_name()),
            type_name,
        ));
    if !structure.fields().is_empty() {
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_fields(typing, project, structure, layout, &ref_type));
    }
    block
}

/// Generates the declaration of the `Ref` type with the `name`, of the `structure`.
fn gen_dec(structure: &Struct, name: &str, type_name: &str) -> RustStruct {
    let mut result: RustStruct = RustStruct::from(name)
        .with_comment(format!(
            " The borrowed encoding of a [{}], read in place.",
            type_name
        ))
        .with_generic("'a")
        .with_field(StructField::from(("bytes", "&'a [u8]")));
    for derive in ["Copy", "Clone", "Debug"] {
        result.add_derive(derive);
    }
    if !is_upper_camel_case(&structure.type_name()) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    result
}

/// Generates the impl block constructing the `ref_type` of the struct with the `type_name`.
fn gen_construction(ref_type: &RustType, type_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("new")
        .with_param(Var::from(("bytes", "&'a [u8]")))
        .with_result(format!("::std::result::Result<Self, {}::DecodeError>", IO));
    let new: Function = Function::from(signature)
        .with_comment(format!(
            " Creates the borrowed encoding of a [{}] from the `bytes`, validating them.",
            type_name
        ))
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(format!(
            " Returns an error if the `bytes` are not a valid encoding of a [{}].",
            type_name
        ))
        .with_access(Access::Public)
        .with_semi(format!("{}::validate(bytes)?", type_name))
        .with_literal("::std::result::Result::Ok(Self { bytes })");

    let signature: Signature = Signature::from("new_unchecked")
        .with_unsafe(true)
        .with_param(Var::from(("bytes", "&'a [u8]")))
        .with_result("Self");
    let new_unchecked: Function = Function::from(signature)
        .with_comment(format!(
            " Creates the borrowed encoding of a [{}] from the `bytes`, without validating them.",
            type_name
        ))
        .with_comment("")
        .with_comment(" # Safety")
        .with_comment(format!(
            " The `bytes` must be a valid encoding of a [{}].",
            type_name
        ))
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

/// Generates the impl block with the encoding of the `ref_type` of the struct with the `type_name`
/// & the `Buf` type with the `buf_name`.
fn gen_encoding(ref_type: &RustType, buf_name: &str, type_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("as_bytes")
        .with_receiver(Receiver::Borrowed)
        .with_result("&'a [u8]");
    let as_bytes: Function = Function::from(signature)
        .with_comment(" Gets the encoding.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true)
        .with_literal("self.bytes");

    let signature: Signature = Signature::from("decode")
        .with_receiver(Receiver::Borrowed)
        .with_result(type_name);
    let decode: Function = Function::from(signature)
        .with_comment(format!(" Decodes the [{}].", type_name))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ {}::decode_unchecked(self.bytes) }}",
            type_name
        ));

    let signature: Signature = Signature::from("to_buf")
        .with_receiver(Receiver::Borrowed)
        .with_result(buf_name);
    let to_buf: Function = Function::from(signature)
        .with_comment(" Copies the encoding into an owned encoding.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!("{} {{ bytes: self.bytes.to_vec() }}", buf_name));

    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Encoding")
        .with_function(as_bytes)
        .with_function(decode)
        .with_function(to_buf)
}

/// Generates the impl block with the field getters of the `ref_type` of the `structure`.
///
/// A variable-size field's data starts at the head's size plus the lengths before it.
fn gen_fields(
    typing: Typing<'_>,
    project: &Project,
    structure: &Struct,
    layout: &StructLayout,
    ref_type: &RustType,
) -> ImplBlock {
    let lengths: Vec<(usize, bool)> = structure
        .fields()
        .iter()
        .zip(layout.fields())
        .filter_map(|(field, field_layout)| match field_layout.value {
            ValueLayout::Length { offset } => Some((offset, field.is_optional())),
            _ => None,
        })
        .collect();
    let length =
        |(offset, is_optional): (usize, bool)| data_length(offset, is_optional, "self.bytes", true);
    let mut lengths_before: usize = 0;
    let mut result: ImplBlock = ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Fields");
    for (field, field_layout) in structure.fields().iter().zip(layout.fields()) {
        let tag: &TypeTag = field.type_tag();
        let mut statements: Vec<String> = Vec::default();
        let (result_type, expr): (RustType, String) = match field_layout.value {
            ValueLayout::Fixed { offset, size } => {
                match fixed_value(project, tag, offset, "self.bytes") {
                    Some(value) if typing.is_converted(tag) => {
                        (typing.value_type(tag), format!("{}.value()", value))
                    }
                    Some(value) => (typing.value_type(tag), value),
                    None => {
                        let path: String = ref_path(tag);
                        (
                            RustType::from(format!("{}<'a>", path)),
                            format!(
                                "unsafe {{ {}::new_unchecked(self.bytes.get_unchecked({}..{})) }}",
                                path,
                                offset,
                                offset + size
                            ),
                        )
                    }
                }
            }
            ValueLayout::Bit(BitLayout { offset, bit }) => {
                (RustType::from("bool"), bit_value(offset, bit, "self.bytes"))
            }
            ValueLayout::Length { offset } => {
                let start: Vec<String> = std::iter::once(layout.head_size().to_string())
                    .chain(lengths[..lengths_before].iter().map(|item| length(*item)))
                    .collect();
                lengths_before += 1;
                statements.push(format!("let start: usize = {}", start.join(" + ")));
                statements.push(format!(
                    "let end: usize = start + {}",
                    length((offset, field.is_optional()))
                ));
                match tag {
                    TypeTag::Special(SpecialType::String) => (
                        RustType::from("&'a str"),
                        "unsafe { ::std::str::from_utf8_unchecked(\
                        self.bytes.get_unchecked(start..end)) }"
                            .to_string(),
                    ),
                    _ => {
                        let path: String = ref_path(tag);
                        (
                            RustType::from(format!("{}<'a>", path)),
                            format!(
                                "unsafe {{ {}::new_unchecked(self.bytes.get_unchecked(start..end)) }}",
                                path
                            ),
                        )
                    }
                }
            }
        };
        let (result_type, expr): (RustType, String) = if field.is_optional() {
            let expr: String = match (field_layout.presence, field_layout.value) {
                (Some(presence), _) => format!(
                    "{}.then(|| {})",
                    bit_value(presence.offset, presence.bit, "self.bytes"),
                    expr
                ),
                (None, ValueLayout::Fixed { offset, .. }) => {
                    optional_enum_value(tag, offset, "self.bytes")
                }
                (None, ValueLayout::Length { offset }) => format!(
                    "(unsafe {{ {}::read_u32_unchecked(self.bytes, {}) }} != u32::MAX).then(|| {})",
                    IO, offset, expr
                ),
                (None, ValueLayout::Bit(_)) => unreachable!("a presence bit"),
            };
            (option(result_type), expr)
        } else {
            (result_type, expr)
        };
        let signature: Signature = Signature::from(ident(&field.field_name()))
            .with_receiver(Receiver::Borrowed)
            .with_result(result_type);
        let mut function: Function = Function::from(signature)
            .with_comment(format!(" Gets the field: `{}`.", field.field_name()))
            .with_attribute("must_use")
            .with_access(Access::Public);
        if !is_snake_case(&field.field_name()) {
            function.add_attribute("allow(non_snake_case)");
        }
        for statement in statements {
            function.add_semi(statement);
        }
        result.add_function(function.with_literal(expr));
    }
    result
}
