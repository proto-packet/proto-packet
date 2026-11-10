use crate::rust::fields::Field;
use crate::rust::gen_buf::buf_name;
use crate::rust::gen_struct_mut::mut_name;
use crate::rust::gen_struct_ref::ref_name;
use crate::rust::gen_view_encoding::{
    ViewEncoding, ViewForm, WithViewFunctions, gen_view_functions, gen_view_overrides,
};
use crate::rust::naming::{ident, is_upper_camel_case};
use crate::rust::typing::Typing;
use crate::rust::values::view_value_type;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, SignatureDec, Trait, WithAccess,
    WithAttributes, WithComments, WithFunctions, WithGenerics, WithReceiver, WithResult,
    WithTraitFunctions,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};

/// Gets the name of the `View` trait of the struct with the schema `type_name`.
///
/// (ex: `FooView` for `Foo`, & `SelfView` for `Self`, whose own name is `Self_`)
pub(crate) fn view_name(type_name: &str) -> String {
    format!("{}View", type_name)
}

/// Generates the `View` trait of the type with the `schema_name` & the Rust `type_name`.
///
/// It has a getter for each field, so generic code can read any form of the type. A nested struct
/// or message is an `impl` of its own `View` trait, since each form has its own type for it. The
/// trait is also implemented for references, since the type's own getters & accessors can return
/// one.
pub(crate) fn gen_view(
    typing: Typing<'_>,
    schema_name: &str,
    fields: &[Field<'_>],
    type_name: &str,
    encoding: &ViewEncoding,
) -> String {
    let name: String = view_name(schema_name);
    let ref_type: RustType = RustType::from(format!("super::{}", ref_name(schema_name)))
        .with_generic(RustType::from("'_"));
    let mut_type: RustType = RustType::from(format!("super::{}", mut_name(schema_name)))
        .with_generic(RustType::from("'_"));
    let buf_type: String = format!("super::{}", buf_name(schema_name));
    let generic: String = format!("V: {}", name);
    let forward = |getter: &str| format!("(**self).{}", getter);
    let own = |getter: &str| format!("self.{}", getter);
    let encoded = |getter: &str| format!("self.as_ref().{}", getter);
    let overrides = |form: ViewForm<'_>| gen_view_overrides(form, encoding.is_level);
    let native: String = format!("super::{}", type_name);
    Block::default()
        .with_statement(gen_trait(
            typing,
            schema_name,
            fields,
            &name,
            type_name,
            encoding,
        ))
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_impl(typing, fields, &name, "&V", forward)
                .with_functions(overrides(ViewForm::Reference))
                .with_generic(generic.as_str()),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_impl(typing, fields, &name, "&mut V", forward)
                .with_functions(overrides(ViewForm::Reference))
                .with_generic(generic.as_str()),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_impl(typing, fields, &name, native.as_str(), own)
                .with_functions(overrides(ViewForm::Native(type_name))),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_impl(typing, fields, &name, ref_type, own).with_functions(overrides(ViewForm::Ref)),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_impl(typing, fields, &name, buf_type.as_str(), encoded)
                .with_functions(overrides(ViewForm::Encoded(&name))),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_impl(typing, fields, &name, mut_type, encoded)
                .with_functions(overrides(ViewForm::Encoded(&name))),
        )
        .to_code()
}

/// Generates the `View` trait with the `name`.
fn gen_trait(
    typing: Typing<'_>,
    schema_name: &str,
    fields: &[Field<'_>],
    name: &str,
    type_name: &str,
    encoding: &ViewEncoding,
) -> Trait {
    let mut result: Trait = Trait::from(name).with_comment(format!(
        " The fields of any form of [{}](super::{}).",
        type_name, type_name
    ));
    if !is_upper_camel_case(schema_name) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    if fields.iter().any(|field| field.is_non_snake_case()) {
        result.add_attribute("allow(non_snake_case)");
    }
    result.set_access(Access::Public);
    for field in fields {
        let signature_dec: SignatureDec = SignatureDec::from(gen_signature(typing, *field))
            .with_comment(format!(" Gets the field: `{}`.", field.name));
        result.add_signature_dec(signature_dec);
    }
    for function in gen_view_functions(encoding) {
        result.add_function(function);
    }
    result
}

/// Generates the impl block of the `View` trait with the `name` for the `target`.
///
/// Each getter's body is the `body` of the call to the field's getter. (ex: `x()`)
fn gen_impl<T, F>(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    name: &str,
    target: T,
    body: F,
) -> ImplBlock
where
    T: Into<RustType>,
    F: Fn(&str) -> String,
{
    let mut result: ImplBlock = ImplBlock::from(target).with_for_trait(name);
    for field in fields {
        let getter: String = format!("{}()", ident(&field.name));
        result.add_function(
            Function::from(gen_signature(typing, *field)).with_literal(body(&getter)),
        );
    }
    result
}

/// Generates the signature of the getter of the `field`.
fn gen_signature(typing: Typing<'_>, field: Field<'_>) -> Signature {
    let value_type: String = view_value_type(typing, field.tag);
    let result: String = if field.is_optional {
        format!("Option<{}>", value_type)
    } else {
        value_type
    };
    Signature::from(ident(&field.name))
        .with_receiver(Receiver::Borrowed)
        .with_result(result)
}
