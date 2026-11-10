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
/// It has a setter for each of the scalar `fields`, & an accessor for each nested struct or message,
/// so generic code can edit any editable form of the type. Every edit is in place & can't fail,
/// so a value that can change its length has no setter. (ex: a string, or a field of a message)
/// The trait is also implemented for mutable references, since the type's own accessors return one.
pub(crate) fn gen_view_mut(
    typing: Typing<'_>,
    schema_name: &str,
    fields: &[Field<'_>],
    type_name: &str,
) -> String {
    let name: String = view_mut_name(schema_name);
    let mut_type: RustType = RustType::from(format!("super::{}", mut_name(schema_name)))
        .with_generic(RustType::from("'_"));
    let buf_type: String = format!("super::{}", buf_name(schema_name));
    let generic: String = format!("V: {}", name);
    Block::default()
        .with_statement(gen_trait(typing, schema_name, fields, &name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(
            gen_impl(typing, fields, &name, "&mut V", Form::Reference)
                .with_generic(generic.as_str()),
        )
        .with_statement(EmptyLine::default())
        .with_statement(gen_impl(
            typing,
            fields,
            &name,
            format!("super::{}", type_name).as_str(),
            Form::Native,
        ))
        .with_statement(EmptyLine::default())
        .with_statement(gen_impl(typing, fields, &name, mut_type, Form::Mut))
        .with_statement(EmptyLine::default())
        .with_statement(gen_impl(
            typing,
            fields,
            &name,
            buf_type.as_str(),
            Form::Buf,
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
            SignatureDec::from(gen_signature(typing, *field)).with_comment(comment);
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
) -> ImplBlock
where
    T: Into<RustType>,
{
    let mut result: ImplBlock = ImplBlock::from(target).with_for_trait(name);
    for field in fields {
        result.add_function(
            Function::from(gen_signature(typing, *field))
                .with_literal(gen_body(typing, *field, form)),
        );
    }
    result
}

/// Generates the signature of the method of the `field`.
fn gen_signature(typing: Typing<'_>, field: Field<'_>) -> Signature {
    if !is_struct(typing, field) {
        return gen_setter_signature(field);
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

/// Generates the statement of the method of the `field` for the `form`.
fn gen_body(typing: Typing<'_>, field: Field<'_>, form: Form) -> String {
    if is_struct(typing, field) {
        return form.gen_accessor_call(&field.accessor_name());
    }
    let setter: String = field.setter_name();
    form.gen_setter_call(&setter)
        .unwrap_or_else(|| format!("self.{}(value);", setter))
}
