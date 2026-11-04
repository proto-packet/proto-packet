use crate::rust::fields::Field;
use crate::rust::gen_buf::buf_name;
use crate::rust::gen_struct_mut::{gen_setter_signature, is_struct, mut_name};
use crate::rust::gen_struct_ref::generated_path;
use crate::rust::gen_view::view_name;
use crate::rust::naming::is_upper_camel_case;
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, SignatureDec, Trait, WithAccess,
    WithAttributes, WithComments, WithFunctions, WithGenerics, WithReceiver, WithResult,
    WithTraitFunctions,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};

/// A form of a type that can be edited.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Form {
    /// A mutable reference to another form.
    Reference,

    /// The type itself.
    Native,

    /// The `Mut` type.
    Mut,

    /// The `Buf` type.
    Buf,
}

impl Form {
    //! Calls

    /// Generates the call to the `accessor` of the form.
    pub(crate) fn gen_accessor_call(self, accessor: &str) -> String {
        match self {
            Self::Reference => format!("(**self).{}()", accessor),
            _ => format!("self.{}()", accessor),
        }
    }

    /// Generates the call to the `setter` of the form, if it takes the trait's `value`.
    ///
    /// The type's own setters return the previous value, & take an owned string.
    pub(crate) fn gen_setter_call(self, setter: &str) -> Option<String> {
        match self {
            Self::Reference => Some(format!("(**self).{}(value)", setter)),
            Self::Mut => Some(format!("self.{}(value)", setter)),
            Self::Buf => Some(format!("self.as_mut().{}(value)", setter)),
            Self::Native => None,
        }
    }
}

/// Gets the name of the `ViewMut` trait of the struct with the schema `type_name`.
///
/// (ex: `FooViewMut` for `Foo`, & `SelfViewMut` for `Self`, whose own name is `Self_`)
pub(crate) fn view_mut_name(type_name: &str) -> String {
    format!("{}ViewMut", type_name)
}

/// Generates the `ViewMut` trait of the type with the `schema_name` & the Rust `type_name`.
///
/// It has a setter for each scalar & string field, & an accessor for each nested struct or message,
/// so generic code can edit any editable form of the type. A string can fail, since the `Mut` type
/// can run out of room, & so can any field of a message, for the same reason, if `all_fallible`. The
/// trait is also implemented for mutable references, since the type's own accessors return one.
pub(crate) fn gen_view_mut(
    typing: Typing<'_>,
    schema_name: &str,
    fields: &[Field<'_>],
    type_name: &str,
    all_fallible: bool,
) -> String {
    let name: String = view_mut_name(schema_name);
    let mut_type: RustType = RustType::from(format!("super::{}", mut_name(schema_name)))
        .with_generic(RustType::from("'_"));
    let buf_type: String = format!("super::{}", buf_name(schema_name));
    let generic: String = format!("V: {}", name);
    Block::default()
        .with_statement(gen_trait(
            typing,
            schema_name,
            fields,
            &name,
            type_name,
            all_fallible,
        ))
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_impl(
                typing,
                fields,
                &name,
                "&mut V",
                Form::Reference,
                all_fallible,
            )
            .with_generic(generic.as_str()),
        )
        .with_statement(EmptyLine::default())
        .with_statement(gen_impl(
            typing,
            fields,
            &name,
            format!("super::{}", type_name).as_str(),
            Form::Native,
            all_fallible,
        ))
        .with_statement(EmptyLine::default())
        .with_statement(gen_impl(
            typing,
            fields,
            &name,
            mut_type,
            Form::Mut,
            all_fallible,
        ))
        .with_statement(EmptyLine::default())
        .with_statement(gen_impl(
            typing,
            fields,
            &name,
            buf_type.as_str(),
            Form::Buf,
            all_fallible,
        ))
        .to_code()
}

/// Generates the `ViewMut` trait with the `name`.
fn gen_trait(
    typing: Typing<'_>,
    schema_name: &str,
    fields: &[Field<'_>],
    name: &str,
    type_name: &str,
    all_fallible: bool,
) -> Trait {
    let mut result: Trait = Trait::from(name)
        .with_comment(format!(
            " The editable fields of any editable form of [{}](super::{}).",
            type_name, type_name
        ))
        .with_supertrait(format!("super::{}", view_name(schema_name)).as_str());
    if !is_upper_camel_case(schema_name) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    if fields.iter().any(|field| field.is_non_snake_case()) {
        result.add_attribute("allow(non_snake_case)");
    }
    result.set_access(Access::Public);
    for field in fields {
        let comment: String = if is_struct(typing, *field) {
            format!(" Gets the mutable field: `{}`.", field.name)
        } else {
            format!(" Sets the field: `{}`.", field.name)
        };
        let signature_dec: SignatureDec =
            SignatureDec::from(gen_signature(typing, *field, all_fallible)).with_comment(comment);
        result.add_signature_dec(signature_dec);
    }
    result
}

/// Generates the impl block of the `ViewMut` trait with the `name` for the `form` type `target`.
fn gen_impl<T>(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    name: &str,
    target: T,
    form: Form,
    all_fallible: bool,
) -> ImplBlock
where
    T: Into<RustType>,
{
    let mut result: ImplBlock = ImplBlock::from(target).with_for_trait(name);
    for field in fields {
        let mut function: Function = Function::from(gen_signature(typing, *field, all_fallible));
        for statement in gen_body(typing, *field, form, all_fallible) {
            function.add_literal(statement);
        }
        result.add_function(function);
    }
    result
}

/// Generates the signature of the method of the `field`.
fn gen_signature(typing: Typing<'_>, field: Field<'_>, all_fallible: bool) -> Signature {
    if !is_struct(typing, field) {
        return gen_setter_signature(field, all_fallible || field.tag.is_string());
    }
    let child: String = format!("impl {}", generated_path(field.tag, "ViewMut"));
    let result: String = if field.is_optional {
        format!("Option<{}>", child)
    } else {
        child
    };
    Signature::from(field.accessor_name())
        .with_receiver(Receiver::BorrowedMut)
        .with_result(result)
}

/// Generates the statements of the method of the `field` for the `form`.
fn gen_body(typing: Typing<'_>, field: Field<'_>, form: Form, all_fallible: bool) -> Vec<String> {
    if is_struct(typing, field) {
        return vec![form.gen_accessor_call(&field.accessor_name())];
    }
    let setter: String = field.setter_name();
    if let Some(call) = form.gen_setter_call(&setter) {
        return vec![call];
    }
    if !field.tag.is_string() {
        let mut statements: Vec<String> = vec![format!("self.{}(value);", setter)];
        if all_fallible {
            statements.push("Ok(())".to_string());
        }
        return statements;
    }
    if field.is_optional {
        return vec![
            "let value: Option<&str> = value.into();".to_string(),
            format!("self.{}(value.map(str::to_string));", setter),
            "Ok(())".to_string(),
        ];
    }
    vec![
        format!("self.{}(value.to_string());", setter),
        "Ok(())".to_string(),
    ]
}
