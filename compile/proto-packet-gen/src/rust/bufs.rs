use crate::rust::decoding::IO;
use crate::rust::naming::{ident, is_upper_camel_case};
use crate::rust::refs::ref_name;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, Struct as RustStruct, StructField,
    Var, WithAccess, WithAttributes, WithComments, WithDerives, WithFunctions, WithReceiver,
    WithResult, WithStructFields, WithUnsafeFlag, WithVarParams,
};
use code_gen::{Block, EmptyLine, WithStatements};
use proto_packet_tree::{Struct, WithTypeName};

/// Gets the Rust name of the `Buf` type of the struct with the schema `type_name`.
///
/// (ex: `FooBuf` for `Foo`, & `SelfBuf` for `Self`, whose own name is `Self_`)
pub(crate) fn buf_name(type_name: &str) -> String {
    ident(&format!("{}Buf", type_name))
}

/// Generates the `Buf` type of the `structure` with the Rust `type_name`.
///
/// It owns the valid encoding of the struct, & reads its fields through its `Ref` type.
/// (see `FORMAT.md`)
pub(crate) fn gen_buf(structure: &Struct, type_name: &str) -> Block {
    let name: String = buf_name(&structure.type_name());
    let ref_name: String = ref_name(&structure.type_name());
    Block::default()
        .with_statement(gen_dec(structure, &name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_construction(&name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_encoding(&name, &ref_name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_from(&name, type_name))
}

/// Generates the declaration of the `Buf` type with the `name`, of the `structure`.
fn gen_dec(structure: &Struct, name: &str, type_name: &str) -> RustStruct {
    let mut result: RustStruct = RustStruct::from(name)
        .with_comment(format!(
            " The owned encoding of a [{}], read in place.",
            type_name
        ))
        .with_field(StructField::from(("bytes", "::std::vec::Vec<u8>")));
    for derive in ["Clone", "Debug"] {
        result.add_derive(derive);
    }
    if !is_upper_camel_case(&structure.type_name()) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    result
}

/// Generates the impl block constructing the `Buf` type with the `name`, of the `type_name`.
fn gen_construction(name: &str, type_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("new")
        .with_param(Var::from(("bytes", "::std::vec::Vec<u8>")))
        .with_result(format!("::std::result::Result<Self, {}::DecodeError>", IO));
    let new: Function = Function::from(signature)
        .with_comment(format!(
            " Creates the owned encoding of a [{}] from the `bytes`, validating them.",
            type_name
        ))
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(format!(
            " Returns an error if the `bytes` are not a valid encoding of a [{}].",
            type_name
        ))
        .with_access(Access::Public)
        .with_semi(format!("{}::validate(&bytes)?", type_name))
        .with_literal("::std::result::Result::Ok(Self { bytes })");

    let signature: Signature = Signature::from("new_unchecked")
        .with_unsafe(true)
        .with_param(Var::from(("bytes", "::std::vec::Vec<u8>")))
        .with_result("Self");
    let new_unchecked: Function = Function::from(signature)
        .with_comment(format!(
            " Creates the owned encoding of a [{}] from the `bytes`, without validating them.",
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

    ImplBlock::from(name)
        .with_comment(" Construction")
        .with_function(new)
        .with_function(new_unchecked)
}

/// Generates the impl block with the encoding of the `Buf` type with the `name`, of the
/// `type_name` with the `Ref` type with the `ref_name`.
fn gen_encoding(name: &str, ref_name: &str, type_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("as_ref")
        .with_receiver(Receiver::Borrowed)
        .with_result(RustType::from(ref_name).with_generic(RustType::from("'_")));
    let as_ref: Function = Function::from(signature)
        .with_comment(" Gets the borrowed encoding, to read the fields.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ {}::new_unchecked(&self.bytes) }}",
            ref_name
        ));

    let signature: Signature = Signature::from("as_bytes")
        .with_receiver(Receiver::Borrowed)
        .with_result("&[u8]");
    let as_bytes: Function = Function::from(signature)
        .with_comment(" Gets the encoding.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal("&self.bytes");

    let signature: Signature = Signature::from("into_bytes")
        .with_receiver(Receiver::Owned)
        .with_result("::std::vec::Vec<u8>");
    let into_bytes: Function = Function::from(signature)
        .with_comment(" Converts the `Buf` into its encoding.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal("self.bytes");

    let signature: Signature = Signature::from("decode")
        .with_receiver(Receiver::Borrowed)
        .with_result(type_name);
    let decode: Function = Function::from(signature)
        .with_comment(format!(" Decodes the [{}].", type_name))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ {}::decode_unchecked(&self.bytes) }}",
            type_name
        ));

    ImplBlock::from(name)
        .with_comment(" Encoding")
        .with_function(as_ref)
        .with_function(as_bytes)
        .with_function(into_bytes)
        .with_function(decode)
}

/// Generates the `From` impl encoding a `type_name` into the `Buf` type with the `name`.
fn gen_from(name: &str, type_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("from")
        .with_param(Var::from(("value", format!("&{}", type_name))))
        .with_result("Self");
    let from: Function = Function::from(signature).with_literal("Self { bytes: value.encode() }");
    ImplBlock::from(name)
        .with_for_trait(format!("::std::convert::From<&{}>", type_name))
        .with_function(from)
}
