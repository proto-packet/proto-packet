use crate::GenError;
use crate::layout::{FieldLayout, StructLayout, ValueLayout};
use crate::rust::encoding::IO;
use crate::rust::fields::Field;
use crate::rust::gen_forms::{gen_new, gen_new_unchecked};
use crate::rust::gen_struct_ref::{generated_path, read_bit, read_entry, ref_name};
use crate::rust::naming::is_upper_camel_case;
use crate::rust::typing::Typing;
use crate::rust::values::setter_value_type;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, Struct as RustStruct, StructField,
    Var, WithAccess, WithAttributes, WithComments, WithDerives, WithFunctions, WithGenerics,
    WithReceiver, WithResult, WithStructFields, WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet_tree::{SpecialType, Struct, TypeTag, WithTypeName};
use std::collections::BTreeSet;

/// Gets the name of the `Mut` type of the struct with the schema `type_name`.
///
/// (ex: `FooMut` for `Foo`, & `SelfMut` for `Self`, whose own name is `Self_`)
pub(crate) fn mut_name(type_name: &str) -> String {
    format!("{}Mut", type_name)
}

/// Checks no two methods of the `Mut` type of the type with the `type_name` have the same name, &
/// no accessor is named like the `Buf` type's `as_mut`.
///
/// (ex: the setter of the field `y_mut` & the accessor of the field `set_y` are both `set_y_mut`)
pub(crate) fn check_mut_names(
    typing: Typing<'_>,
    type_name: &str,
    fields: &[Field<'_>],
) -> Result<(), GenError> {
    let mut names: BTreeSet<String> = BTreeSet::from(["as_mut".to_string()]);
    for field in fields.iter().filter(|field| !field.tag.is_collection()) {
        if !names.insert(method_name(typing, *field)) {
            return Err(GenError::FieldNameConflict {
                type_name: type_name.to_string(),
                field_name: field.name.to_string(),
            });
        }
    }
    Ok(())
}

/// Checks if the `field` is a nested struct.
pub(crate) fn is_struct(typing: Typing<'_>, field: Field<'_>) -> bool {
    matches!(field.tag, TypeTag::Named(_)) && !typing.is_enum(field.tag)
}

/// Gets the name of the method of the `field`. (ex: `set_x`, or `x_mut` for a nested struct)
fn method_name(typing: Typing<'_>, field: Field<'_>) -> String {
    if is_struct(typing, field) {
        field.accessor_name()
    } else {
        field.setter_name()
    }
}

/// Generates the `Mut` type of the `structure` with the Rust `type_name`.
///
/// It edits the valid encoding of the struct in place, & every edit keeps it valid. A setter can't
/// fail, so only a fixed-size field has one. Its file imports nothing & declares only the `Mut`
/// type, so the prelude names are never shadowed.
pub(crate) fn gen_struct_mut(
    typing: Typing<'_>,
    structure: &Struct,
    fields: &[Field<'_>],
    layout: &StructLayout,
    type_name: &str,
) -> String {
    let name: String = mut_name(&structure.type_name());
    let ref_name: String = ref_name(&structure.type_name());
    let any_lifetime: RustType = RustType::from(name.as_str()).with_generic(RustType::from("'_"));
    let mut block: Block = Block::default()
        .with_statement(gen_mut_dec(&structure.type_name(), &name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_mut_construction(&name, &ref_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_mut_properties(&any_lifetime, &ref_name));
    if fields.iter().any(|field| !field.tag.is_collection()) {
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_fields(typing, fields, layout, &any_lifetime));
    }
    block.to_code()
}

/// Generates the declaration of the `Mut` type with the `name`.
pub(crate) fn gen_mut_dec(schema_name: &str, name: &str, type_name: &str) -> RustStruct {
    let mut result: RustStruct = RustStruct::from(name)
        .with_comment(format!(
            " A mutable view of an encoded [{}](super::{}).",
            type_name, type_name
        ))
        .with_generic("'a")
        .with_field(StructField::from(("bytes", "&'a mut [u8]")));
    result.add_derive("Debug");
    if !is_upper_camel_case(schema_name) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    result
}

/// Generates the impl block constructing the `Mut` type with the `name`.
pub(crate) fn gen_mut_construction(name: &str, ref_name: &str) -> ImplBlock {
    let new: Function = gen_new("mut", "&'a mut [u8]")
        .with_semi(format!("super::{}::new(bytes)?", ref_name))
        .with_literal("Ok(Self { bytes })");

    ImplBlock::from(RustType::from(name).with_generic(RustType::from("'a")))
        .with_generic("'a")
        .with_comment(" Construction")
        .with_function(new)
        .with_function(gen_new_unchecked("mut", "&'a mut [u8]"))
}

/// Generates the impl block with the properties of the `mut_type`.
pub(crate) fn gen_mut_properties(mut_type: &RustType, ref_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("as_ref")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("super::{}<'_>", ref_name));
    let as_ref: Function = Function::from(signature)
        .with_comment(" Gets the ref of the bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ super::{}::new_unchecked(self.bytes) }}",
            ref_name
        ));

    let signature: Signature = Signature::from("as_bytes")
        .with_receiver(Receiver::Borrowed)
        .with_result("&[u8]");
    let as_bytes: Function = Function::from(signature)
        .with_comment(" Gets the encoded bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal("self.bytes");

    ImplBlock::from(mut_type.clone())
        .with_comment(" Properties")
        .with_function(as_ref)
        .with_function(as_bytes)
}

/// Generates the impl block with the method of each of the `fields`.
fn gen_fields(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    layout: &StructLayout,
    mut_type: &RustType,
) -> ImplBlock {
    let mut result: ImplBlock = ImplBlock::from(mut_type.clone()).with_comment(" Fields");
    for (field, field_layout) in fields.iter().zip(&layout.fields) {
        if field.tag.is_collection() || field.tag.is_string() {
            continue;
        }
        let mut function: Function = if is_struct(typing, *field) {
            gen_child(*field, *field_layout, layout, "self.bytes", "self.bytes")
        } else {
            gen_setter(typing, *field, *field_layout)
        };
        if field.is_non_snake_case() {
            function.add_attribute("allow(non_snake_case)");
        }
        result.add_function(function);
    }
    result
}

/// Generates the accessor of the nested struct `field`, to edit it in place.
///
/// The `bytes` & `bytes_mut` expressions are the type's bytes, as a shared & a mutable slice.
pub(crate) fn gen_child(
    field: Field<'_>,
    field_layout: FieldLayout,
    layout: &StructLayout,
    bytes: &str,
    bytes_mut: &str,
) -> Function {
    let child: String = generated_path(field.tag, "Mut");
    let child_bytes: String = match field_layout.value {
        ValueLayout::Fixed { offset, size } => {
            format!(
                "{}.get_unchecked_mut({}..{})",
                bytes_mut,
                offset,
                offset + size
            )
        }
        ValueLayout::Entry { index } => {
            read_entry("read_tail_entry_mut_unchecked", layout, index, bytes_mut)
        }
        ValueLayout::Bit(_) => unreachable!("a struct is not a bit"),
    };
    let value: String = format!("unsafe {{ {}::new_unchecked({}) }}", child, child_bytes);
    let (result, value): (String, String) = match field_layout.presence {
        Some(presence) => (
            format!("Option<{}<'_>>", child),
            format!(
                "if {} {{ Some({}) }} else {{ None }}",
                read_bit(presence, bytes),
                value
            ),
        ),
        None => (format!("{}<'_>", child), value),
    };
    let signature: Signature = Signature::from(field.accessor_name())
        .with_receiver(Receiver::BorrowedMut)
        .with_result(result);
    Function::from(signature)
        .with_comment(format!(" Gets the mut of the field: `{}`.", field.name))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(value)
}

/// Generates the signature of the setter of the scalar `field`.
pub(crate) fn gen_setter_signature(field: Field<'_>) -> Signature {
    let param: String = if field.is_optional {
        format!("impl Into<Option<{}>>", setter_value_type(field.tag))
    } else {
        setter_value_type(field.tag)
    };
    Signature::from(field.setter_name())
        .with_receiver(Receiver::BorrowedMut)
        .with_param(Var::from(("value", param.as_str())))
}

/// Generates the setter of the scalar `field`.
fn gen_setter(typing: Typing<'_>, field: Field<'_>, field_layout: FieldLayout) -> Function {
    let mut function: Function = Function::from(gen_setter_signature(field))
        .with_comment(format!(" Sets the field: `{}`.", field.name))
        .with_access(Access::Public);
    if field.is_optional {
        function.add_semi(format!(
            "let value: Option<{}> = value.into()",
            setter_value_type(field.tag)
        ));
    }
    function.add_literal(gen_write(typing, field, field_layout));
    if let Some(presence) = field_layout.presence {
        function.add_literal(format!(
            "unsafe {{ {}::write_bit_unchecked(self.bytes, {}, {}, value.is_some()) }};",
            IO, presence.offset, presence.bit
        ));
    }
    function
}

/// Generates the statement writing the `value` of the `field`.
///
/// An absent field is written as zeros.
fn gen_write(typing: Typing<'_>, field: Field<'_>, field_layout: FieldLayout) -> String {
    let value: &str = if field.is_optional {
        "value.unwrap_or_default()"
    } else {
        "value"
    };
    let write = |function: &str, offset: usize, value: &str| -> String {
        format!(
            "unsafe {{ {}::{}(self.bytes, {}, {}) }};",
            IO, function, offset, value
        )
    };
    match (field_layout.value, field.tag) {
        (ValueLayout::Bit(bit), _) => format!(
            "unsafe {{ {}::write_bit_unchecked(self.bytes, {}, {}, {}) }};",
            IO, bit.offset, bit.bit, value
        ),
        (ValueLayout::Fixed { offset, .. }, TypeTag::Primitive(primitive)) => write(
            &format!("write_{}_unchecked", primitive.name()),
            offset,
            value,
        ),
        (ValueLayout::Fixed { offset, .. }, TypeTag::Special(SpecialType::Uuid)) => write(
            "write_bytes_unchecked",
            offset,
            &format!("*{}.as_bytes()", value),
        ),
        (ValueLayout::Fixed { offset, .. }, _) if typing.is_enum(field.tag) => {
            let tag_number: &str = if field.is_optional {
                "value.map_or(0, |value| value.tag_number())"
            } else {
                "value.tag_number()"
            };
            write("write_u16_unchecked", offset, tag_number)
        }
        _ => unreachable!("only scalar fields have a setter"),
    }
}

/// Generates the impl block of the `Buf` type with the `name`, with an accessor for each nested
/// struct or message of the `fields`.
///
/// A `Buf` type can't lend these through a temporary `Mut` type, so it has its own.
pub(crate) fn gen_buf_accessors(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    layout: &StructLayout,
    name: &str,
) -> Option<ImplBlock> {
    if !fields.iter().any(|field| is_struct(typing, *field)) {
        return None;
    }
    let mut result: ImplBlock = ImplBlock::from(name).with_comment(" Fields");
    for (field, field_layout) in fields.iter().zip(&layout.fields) {
        if is_struct(typing, *field) {
            let mut function: Function = gen_child(
                *field,
                *field_layout,
                layout,
                "self.bytes.as_slice()",
                "self.bytes.as_mut_slice()",
            );
            if field.is_non_snake_case() {
                function.add_attribute("allow(non_snake_case)");
            }
            result.add_function(function);
        }
    }
    Some(result)
}
