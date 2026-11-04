use crate::rust::encoding::IO;
use crate::rust::gen_struct_mut::mut_name;
use crate::rust::gen_struct_ref::ref_name;
use crate::rust::naming::is_upper_camel_case;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, Signature, Struct as RustStruct, StructField, Var,
    WithAccess, WithAttributes, WithComments, WithDerives, WithFunctions, WithReceiver, WithResult,
    WithStructFields, WithUnsafeFlag, WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};

/// The type of a `Buf` type's bytes.
const BYTES: &str = "Vec<u8>";

/// Gets the name of the `Buf` type of the struct with the schema `type_name`.
///
/// (ex: `FooBuf` for `Foo`, & `SelfBuf` for `Self`, whose own name is `Self_`)
pub(crate) fn buf_name(type_name: &str) -> String {
    format!("{}Buf", type_name)
}

/// Generates the `Buf` type of the type with the `schema_name` & the Rust `type_name`.
///
/// It owns the valid encoding of the type, & reads its fields through its `Ref` type. Its file
/// imports nothing & declares only the `Buf` type, so the prelude names are never shadowed.
pub(crate) fn gen_buf(schema_name: &str, type_name: &str, accessors: Option<ImplBlock>) -> String {
    let name: String = buf_name(schema_name);
    let ref_name: String = ref_name(schema_name);
    let mut_name: String = mut_name(schema_name);
    let mut block: Block = Block::default()
        .with_statement(gen_dec(schema_name, &name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_construction(&name, &ref_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_from(&name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_properties(&name, &ref_name, &mut_name));
    if let Some(accessors) = accessors {
        block.add_statement(EmptyLine::default());
        block.add_statement(accessors);
    }
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_decoding(&name, type_name));
    block.to_code()
}

/// Generates the declaration of the `Buf` type with the `name`.
fn gen_dec(schema_name: &str, name: &str, type_name: &str) -> RustStruct {
    let mut result: RustStruct = RustStruct::from(name)
        .with_comment(format!(
            " An owned encoded [{}](super::{}).",
            type_name, type_name
        ))
        .with_field(StructField::from(("bytes", BYTES)));
    for derive in ["Clone", "Debug"] {
        result.add_derive(derive);
    }
    if !is_upper_camel_case(schema_name) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    result
}

/// Generates the impl block constructing the `Buf` type with the `name`.
fn gen_construction(name: &str, ref_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("new")
        .with_param(Var::from(("bytes", BYTES)))
        .with_result(format!("Result<Self, {}::DecodeError>", IO));
    let new: Function = Function::from(signature)
        .with_comment(" Creates a new buf of the `bytes`.")
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(" Returns an error if the `bytes` are not a valid encoding.")
        .with_access(Access::Public)
        .with_semi(format!("super::{}::new(&bytes)?", ref_name))
        .with_literal("Ok(Self { bytes })");

    let signature: Signature = Signature::from("new_unchecked")
        .with_unsafe(true)
        .with_param(Var::from(("bytes", BYTES)))
        .with_result("Self");
    let new_unchecked: Function = Function::from(signature)
        .with_comment(" Creates a new buf of the `bytes`, without validating them.")
        .with_comment("")
        .with_comment(" # Safety")
        .with_comment(" The `bytes` must be a valid encoding.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true)
        .with_literal("Self { bytes }");

    ImplBlock::from(name)
        .with_comment(" Construction")
        .with_function(new)
        .with_function(new_unchecked)
}

/// Generates the `From` impl encoding the struct with the `type_name`.
fn gen_from(name: &str, type_name: &str) -> ImplBlock {
    let value: String = format!("&super::{}", type_name);
    let signature: Signature = Signature::from("from")
        .with_param(Var::from(("value", value.as_str())))
        .with_result("Self");
    let from: Function = Function::from(signature).with_literal("Self { bytes: value.encode() }");
    ImplBlock::from(name)
        .with_for_trait(format!("From<{}>", value).as_str())
        .with_function(from)
}

/// Generates the impl block with the properties of the `Buf` type with the `name`.
fn gen_properties(name: &str, ref_name: &str, mut_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("as_ref")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("super::{}<'_>", ref_name));
    let as_ref: Function = Function::from(signature)
        .with_comment(" Gets the ref of the bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ super::{}::new_unchecked(&self.bytes) }}",
            ref_name
        ));

    let signature: Signature = Signature::from("as_mut")
        .with_receiver(Receiver::BorrowedMut)
        .with_result(format!("super::{}<'_>", mut_name));
    let as_mut: Function = Function::from(signature)
        .with_comment(" Gets the mut of the bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ super::{}::new_unchecked(&mut self.bytes) }}",
            mut_name
        ));

    let signature: Signature = Signature::from("as_bytes")
        .with_receiver(Receiver::Borrowed)
        .with_result("&[u8]");
    let as_bytes: Function = Function::from(signature)
        .with_comment(" Gets the encoded bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal("&self.bytes");

    let signature: Signature = Signature::from("into_bytes")
        .with_receiver(Receiver::Owned)
        .with_result(BYTES);
    let into_bytes: Function = Function::from(signature)
        .with_comment(" Converts the buf into its encoded bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal("self.bytes");

    ImplBlock::from(name)
        .with_comment(" Properties")
        .with_function(as_ref)
        .with_function(as_mut)
        .with_function(as_bytes)
        .with_function(into_bytes)
}

/// Generates the impl block decoding the `Buf` type with the `name`.
fn gen_decoding(name: &str, type_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("decode")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("super::{}", type_name));
    let decode: Function = Function::from(signature)
        .with_comment(format!(
            " Decodes the [{}](super::{}).",
            type_name, type_name
        ))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal("self.as_ref().decode()");
    ImplBlock::from(name)
        .with_comment(" Decoding")
        .with_function(decode)
}
