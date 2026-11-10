use crate::rust::encoding::IO;
use crate::rust::gen_struct_mut::{
    gen_mut_construction, gen_mut_dec, gen_mut_properties, mut_name,
};
use crate::rust::gen_struct_ref::{generated_path, ref_name};
use crate::rust::gen_variant::Case;
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, WithAccess, WithAttributes,
    WithComments, WithFunctions, WithReceiver, WithResult,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};

/// Generates the `Mut` type of the variant with the `schema_name` & the Rust `type_name`.
///
/// It edits the valid encoding of the variant in place, & every edit keeps it valid. It has no
/// setters, since another case can have another length, so it only reaches its nested values.
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
    if cases.iter().any(|case| case.is_nested(typing)) {
        let mut fields: ImplBlock = ImplBlock::from(any_lifetime).with_comment(" Cases");
        for case in cases.iter().filter(|case| case.is_nested(typing)) {
            fields.add_function(gen_child(*case, "self.bytes", "self.bytes"));
        }
        block.add_statement(EmptyLine::default());
        block.add_statement(fields);
    }
    block.to_code()
}

/// Generates the accessor of the nested `case`, to edit it in place.
///
/// The `bytes` & `bytes_mut` expressions are the type's bytes, as a shared & a mutable slice.
fn gen_child(case: Case<'_>, bytes: &str, bytes_mut: &str) -> Function {
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
            "Some(unsafe {{ {}::new_unchecked({}.get_unchecked_mut(2..)) }})",
            child, bytes_mut
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
            result.add_function(gen_child(
                *case,
                "self.bytes.as_slice()",
                "self.bytes.as_mut_slice()",
            ));
        }
    }
    Some(result)
}
