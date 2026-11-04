use crate::rust::encoding::IO;
use crate::rust::gen_message_mut::gen_bytes;
use crate::rust::gen_struct_mut::{
    gen_mut_construction, gen_mut_dec, gen_mut_properties, mut_name,
};
use crate::rust::gen_struct_ref::{generated_path, ref_name};
use crate::rust::gen_variant::Case;
use crate::rust::naming::type_path;
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, Var, WithAccess, WithAttributes,
    WithComments, WithFunctions, WithReceiver, WithResult, WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet_tree::{SpecialType, TypeTag};

/// Generates the `Mut` type of the variant with the `schema_name` & the Rust `type_name`.
///
/// It edits the valid encoding of the variant in place, & every edit keeps it valid. A variant has
/// no length of its own, so a string must fill the variant, & any other case must fit in it.
pub(crate) fn gen_variant_mut(
    typing: Typing<'_>,
    schema_name: &str,
    cases: &[Case<'_>],
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
    if !cases.is_empty() {
        let mut fields: ImplBlock = ImplBlock::from(any_lifetime).with_comment(" Cases");
        for case in cases {
            fields.add_function(if case.is_nested(typing) {
                gen_child(*case, "self.bytes")
            } else {
                gen_setter(typing, *case)
            });
        }
        block.add_statement(EmptyLine::default());
        block.add_statement(fields);
    }
    block.to_code()
}

/// Generates the accessor of the nested `case`, to edit it in place.
///
/// The `bytes` expression is the type's bytes, as a shared slice.
fn gen_child(case: Case<'_>, bytes: &str) -> Function {
    let child: String = generated_path(case.tag, "Mut");
    let signature: Signature = Signature::from(case.accessor_name())
        .with_receiver(Receiver::BorrowedMut)
        .with_result(format!("Option<{}<'_>>", child));
    Function::from(signature)
        .with_comment(format!(" Gets the mut of the case: `{}`.", case.name))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "if unsafe {{ {}::read_u16_unchecked({}, 0) }} != {} {{ return None; }}",
            IO, bytes, case.number
        ))
        .with_literal(format!(
            "Some(unsafe {{ {}::new_unchecked(self.bytes.get_unchecked_mut(2..)) }})",
            child
        ))
}

/// Generates the impl block of the `Buf` type with the `name`, with an accessor for each nested
/// case of the `cases`.
///
/// A `Buf` type can't lend these through a temporary `Mut` type, so it has its own.
pub(crate) fn gen_variant_buf_accessors(
    typing: Typing<'_>,
    cases: &[Case<'_>],
    name: &str,
) -> Option<ImplBlock> {
    if !cases.iter().any(|case| case.is_nested(typing)) {
        return None;
    }
    let mut result: ImplBlock = ImplBlock::from(name).with_comment(" Cases");
    for case in cases {
        if case.is_nested(typing) {
            result.add_function(gen_child(*case, "&self.bytes"));
        }
    }
    Some(result)
}

/// Gets the type of a value of the scalar or string `case`.
fn value_type(case: Case<'_>) -> String {
    match case.tag {
        TypeTag::Primitive(primitive) => primitive.name().to_string(),
        TypeTag::Special(SpecialType::Uuid) => "proto_packet::types::Uuid".to_string(),
        TypeTag::Special(SpecialType::String) => "&str".to_string(),
        TypeTag::Named(name) => type_path(name.to_ref()),
    }
}

/// Generates the signature of the setter of the scalar or string `case`.
pub(crate) fn gen_setter_signature(case: Case<'_>) -> Signature {
    Signature::from(case.setter_name())
        .with_receiver(Receiver::BorrowedMut)
        .with_param(Var::from(("value", value_type(case).as_str())))
        .with_result(format!("Result<(), {}::CapacityError>", IO))
}

/// Generates the setter of the scalar or string `case`.
fn gen_setter(typing: Typing<'_>, case: Case<'_>) -> Function {
    let (bytes, exact): (String, bool) = match case.tag {
        TypeTag::Special(SpecialType::String) => ("value.as_bytes()".to_string(), true),
        TypeTag::Special(SpecialType::Uuid) => ("value.as_bytes()".to_string(), false),
        _ => (format!("&{}", gen_bytes(typing, case.tag)), false),
    };
    Function::from(gen_setter_signature(case))
        .with_comment(format!(" Sets the case: `{}`.", case.name))
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(" Returns an error, leaving the variant unchanged, if the value doesn't fit.")
        .with_access(Access::Public)
        .with_literal(format!(
            "{}::replace_case(self.bytes, {}, {}, {})",
            IO, case.number, bytes, exact
        ))
}
