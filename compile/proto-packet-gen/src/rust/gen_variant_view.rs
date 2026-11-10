use crate::rust::gen_buf::buf_name;
use crate::rust::gen_struct_mut::mut_name;
use crate::rust::gen_struct_ref::{generated_path, ref_name};
use crate::rust::gen_variant::Case;
use crate::rust::gen_view::view_name;
use crate::rust::gen_view_encoding::{
    ViewEncoding, ViewForm, WithViewFunctions, gen_variant_view_encoding, gen_view_functions,
    gen_view_overrides,
};
use crate::rust::gen_view_mut::{Form, view_mut_name};
use crate::rust::naming::is_upper_camel_case;
use crate::rust::typing::Typing;
use crate::rust::values::view_value_type;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, SignatureDec, Trait, WithAccess,
    WithAttributes, WithComments, WithFunctions, WithGenerics, WithReceiver, WithResult,
    WithTraitFunctions,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};

/// Generates the `View` trait of the variant with the `schema_name` & the Rust `type_name`.
///
/// It has the tag number & a getter for each case, at most one of which is `Some`, so generic code
/// can read any form of the variant. The trait is also implemented for references, since the
/// variant's own getters & accessors can return one.
pub(crate) fn gen_variant_view(
    typing: Typing<'_>,
    schema_name: &str,
    cases: &[Case<'_>],
    type_name: &str,
) -> String {
    let name: String = view_name(schema_name);
    let ref_type: RustType = RustType::from(format!("super::{}", ref_name(schema_name)))
        .with_generic(RustType::from("'_"));
    let mut_type: RustType = RustType::from(format!("super::{}", mut_name(schema_name)))
        .with_generic(RustType::from("'_"));
    let buf_type: String = format!("super::{}", buf_name(schema_name));
    let generic: String = format!("V: {}", name);

    let mut result: Trait = Trait::from(name.as_str()).with_comment(format!(
        " The cases of any form of [{}](super::{}).",
        type_name, type_name
    ));
    if !is_upper_camel_case(schema_name) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    result.add_signature_dec(
        SignatureDec::from(tag_number_signature())
            .with_comment(" Gets the tag number of the case."),
    );
    for case in cases {
        result.add_signature_dec(
            SignatureDec::from(gen_getter_signature(typing, *case))
                .with_comment(format!(" Gets the value of the case: `{}`.", case.name)),
        );
    }
    let encoding: ViewEncoding = gen_variant_view_encoding(typing, cases);
    for function in gen_view_functions(&encoding) {
        result.add_function(function);
    }

    let forward = |call: &str| format!("(**self).{}", call);
    let own = |call: &str| format!("self.{}", call);
    let encoded = |call: &str| format!("self.as_ref().{}", call);
    let overrides = |form: ViewForm<'_>| gen_view_overrides(form, true);
    let native: String = format!("super::{}", type_name);
    Block::default()
        .with_statement(result)
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_view_impl(typing, cases, &name, "&V", forward)
                .with_functions(overrides(ViewForm::Reference))
                .with_generic(generic.as_str()),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_view_impl(typing, cases, &name, "&mut V", forward)
                .with_functions(overrides(ViewForm::Reference))
                .with_generic(generic.as_str()),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_view_impl(typing, cases, &name, native.as_str(), own)
                .with_functions(overrides(ViewForm::Native(type_name))),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_view_impl(typing, cases, &name, ref_type, own)
                .with_functions(overrides(ViewForm::Ref)),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_view_impl(typing, cases, &name, buf_type.as_str(), encoded)
                .with_functions(overrides(ViewForm::Encoded(&name))),
        )
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_view_impl(typing, cases, &name, mut_type, encoded)
                .with_functions(overrides(ViewForm::Encoded(&name))),
        )
        .to_code()
}

/// Generates the impl block of the `View` trait with the `name` for the `target`.
///
/// Each body is the `body` of the call. (ex: `as_circle()`)
fn gen_view_impl<T, F>(
    typing: Typing<'_>,
    cases: &[Case<'_>],
    name: &str,
    target: T,
    body: F,
) -> ImplBlock
where
    T: Into<RustType>,
    F: Fn(&str) -> String,
{
    let mut result: ImplBlock = ImplBlock::from(target)
        .with_for_trait(name)
        .with_function(Function::from(tag_number_signature()).with_literal(body("tag_number()")));
    for case in cases {
        let call: String = format!("{}()", case.getter_name());
        result.add_function(
            Function::from(gen_getter_signature(typing, *case)).with_literal(body(&call)),
        );
    }
    result
}

/// Generates the signature of the tag number getter.
fn tag_number_signature() -> Signature {
    Signature::from("tag_number")
        .with_receiver(Receiver::Borrowed)
        .with_result("u16")
}

/// Generates the signature of the getter of the `case`.
fn gen_getter_signature(typing: Typing<'_>, case: Case<'_>) -> Signature {
    let value_type: String = view_value_type(typing, case.tag);
    Signature::from(case.getter_name())
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("Option<{}>", value_type))
}

/// Generates the `ViewMut` trait of the variant with the `schema_name` & the Rust `type_name`.
///
/// It has a setter for each scalar & string case, & an accessor for each nested case, so generic
/// code can edit any editable form of the variant. The trait is also implemented for mutable
/// references, since the variant's own accessors return one.
pub(crate) fn gen_variant_view_mut(
    schema_name: &str,
    cases: &[Case<'_>],
    type_name: &str,
) -> String {
    let name: String = view_mut_name(schema_name);
    let mut_type: RustType = RustType::from(format!("super::{}", mut_name(schema_name)))
        .with_generic(RustType::from("'_"));
    let buf_type: String = format!("super::{}", buf_name(schema_name));
    let generic: String = format!("V: {}", name);

    let mut result: Trait = Trait::from(name.as_str())
        .with_comment(format!(
            " The editable cases of any editable form of [{}](super::{}).",
            type_name, type_name
        ))
        .with_supertrait(format!("super::{}", view_name(schema_name)).as_str());
    if !is_upper_camel_case(schema_name) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    for case in cases {
        result.add_signature_dec(
            SignatureDec::from(gen_method_signature(*case)).with_comment(format!(
                " Gets the mutable value of the case: `{}`.",
                case.name
            )),
        );
    }

    Block::default()
        .with_statement(result)
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_view_mut_impl(cases, &name, "&mut V", Form::Reference)
                .with_generic(generic.as_str()),
        )
        .with_statement(EmptyLine::default())
        .with_statement(gen_view_mut_impl(
            cases,
            &name,
            format!("super::{}", type_name).as_str(),
            Form::Native,
        ))
        .with_statement(EmptyLine::default())
        .with_statement(gen_view_mut_impl(cases, &name, mut_type, Form::Mut))
        .with_statement(EmptyLine::default())
        .with_statement(gen_view_mut_impl(
            cases,
            &name,
            buf_type.as_str(),
            Form::Buf,
        ))
        .to_code()
}

/// Generates the impl block of the `ViewMut` trait with the `name` for the `form` type `target`.
fn gen_view_mut_impl<T>(cases: &[Case<'_>], name: &str, target: T, form: Form) -> ImplBlock
where
    T: Into<RustType>,
{
    let mut result: ImplBlock = ImplBlock::from(target).with_for_trait(name);
    for case in cases {
        result.add_function(
            Function::from(gen_method_signature(*case))
                .with_literal(form.gen_accessor_call(&case.accessor_name())),
        );
    }
    result
}

/// Generates the signature of the accessor of the nested `case`.
fn gen_method_signature(case: Case<'_>) -> Signature {
    Signature::from(case.accessor_name())
        .with_receiver(Receiver::BorrowedMut)
        .with_result(format!(
            "Option<impl {}>",
            generated_path(case.tag, "ViewMut")
        ))
}
