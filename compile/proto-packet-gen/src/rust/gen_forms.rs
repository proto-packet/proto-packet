//! The functions shared by the encoded forms of a type: its `Ref`, `Mut` & `Buf` types.

use crate::rust::encoding::IO;
use code_gen::WithStatements;
use code_gen::rust::{
    Access, Function, Receiver, Signature, Var, WithAccess, WithAttributes, WithComments,
    WithReceiver, WithResult, WithUnsafeFlag, WithVarParams,
};

/// Generates the start of the validating constructor of the `form`, taking the `bytes` type.
///
/// (ex: the `form` is `ref`)
pub(crate) fn gen_new(form: &str, bytes: &str) -> Function {
    let signature: Signature = Signature::from("new")
        .with_param(Var::from(("bytes", bytes)))
        .with_result(format!("Result<Self, {}::DecodeError>", IO));
    Function::from(signature)
        .with_comment(format!(" Creates a new {} of the `bytes`.", form))
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(" Returns an error if the `bytes` are not a valid encoding.")
        .with_access(Access::Public)
}

/// Generates the constructor of the `form` that skips validation, taking the `bytes` type.
pub(crate) fn gen_new_unchecked(form: &str, bytes: &str) -> Function {
    let signature: Signature = Signature::from("new_unchecked")
        .with_unsafe(true)
        .with_param(Var::from(("bytes", bytes)))
        .with_result("Self");
    Function::from(signature)
        .with_comment(format!(
            " Creates a new {} of the `bytes`, without validating them.",
            form
        ))
        .with_comment("")
        .with_comment(" # Safety")
        .with_comment(" The `bytes` must be a valid encoding.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true)
        .with_literal("Self { bytes }")
}

/// Generates the function of a `Ref` type copying its bytes into the `Buf` type with the `name`.
pub(crate) fn gen_to_buf(name: &str) -> Function {
    let signature: Signature = Signature::from("to_buf")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("super::{}", name));
    Function::from(signature)
        .with_comment(" Copies the bytes into an owned buf.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ super::{}::new_unchecked(self.bytes.to_vec()) }}",
            name
        ))
}
