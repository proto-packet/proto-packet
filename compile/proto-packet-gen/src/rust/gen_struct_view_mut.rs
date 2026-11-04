use crate::rust::fields::Field;
use crate::rust::gen_struct_buf::buf_name;
use crate::rust::gen_struct_mut::{gen_setter_signature, is_struct, mut_name};
use crate::rust::gen_struct_ref::generated_path;
use crate::rust::gen_struct_view::view_name;
use crate::rust::naming::is_upper_camel_case;
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, RustType, Signature, SignatureDec, Trait, WithAccess,
    WithAttributes, WithComments, WithFunctions, WithGenerics, WithReceiver, WithResult,
    WithTraitFunctions,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet_tree::{SpecialType, Struct, TypeTag, WithTypeName};

/// A form of a struct that can be edited.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Form {
    /// A mutable reference to another form.
    Reference,

    /// The struct itself.
    Native,

    /// The `Mut` type.
    Mut,

    /// The `Buf` type.
    Buf,
}

/// Gets the name of the `ViewMut` trait of the struct with the schema `type_name`.
///
/// (ex: `FooViewMut` for `Foo`, & `SelfViewMut` for `Self`, whose own name is `Self_`)
pub(crate) fn view_mut_name(type_name: &str) -> String {
    format!("{}ViewMut", type_name)
}

/// Generates the `ViewMut` trait of the `structure` with the Rust `type_name`.
///
/// It has a setter for each scalar & string field, & an accessor for each nested struct, so generic
/// code can edit any editable form of the struct. A string can fail, since the `Mut` type can run
/// out of room. The trait is also implemented for mutable references, since the struct's own
/// accessors return one.
pub(crate) fn gen_struct_view_mut(
    typing: Typing<'_>,
    structure: &Struct,
    fields: &[Field<'_>],
    type_name: &str,
) -> String {
    let name: String = view_mut_name(&structure.type_name());
    let mut_type: RustType = RustType::from(format!("super::{}", mut_name(&structure.type_name())))
        .with_generic(RustType::from("'_"));
    let buf_type: String = format!("super::{}", buf_name(&structure.type_name()));
    let generic: String = format!("V: {}", name);
    Block::default()
        .with_statement(gen_trait(typing, structure, fields, &name, type_name))
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
    structure: &Struct,
    fields: &[Field<'_>],
    name: &str,
    type_name: &str,
) -> Trait {
    let mut result: Trait = Trait::from(name)
        .with_comment(format!(
            " The editable fields of any editable form of [{}](super::{}).",
            type_name, type_name
        ))
        .with_supertrait(format!("super::{}", view_name(&structure.type_name())).as_str());
    if !is_upper_camel_case(&structure.type_name()) {
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
        let mut function: Function = Function::from(gen_signature(typing, *field));
        for statement in gen_body(typing, *field, form) {
            function.add_literal(statement);
        }
        result.add_function(function);
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

/// Generates the statements of the method of the `field` for the `form`.
///
/// The struct's own setters return the previous value, & take an owned string.
fn gen_body(typing: Typing<'_>, field: Field<'_>, form: Form) -> Vec<String> {
    if is_struct(typing, field) {
        let receiver: &str = if form == Form::Reference {
            "(**self)"
        } else {
            "self"
        };
        return vec![format!("{}.{}()", receiver, field.accessor_name())];
    }
    let setter: String = field.setter_name();
    let is_string: bool = *field.tag == TypeTag::Special(SpecialType::String);
    match form {
        Form::Reference => vec![format!("(**self).{}(value)", setter)],
        Form::Mut => vec![format!("self.{}(value)", setter)],
        Form::Buf => vec![format!("self.as_mut().{}(value)", setter)],
        Form::Native if !is_string => vec![format!("self.{}(value);", setter)],
        Form::Native if field.is_optional => vec![
            "let value: Option<&str> = value.into();".to_string(),
            format!("self.{}(value.map(str::to_string));", setter),
            "Ok(())".to_string(),
        ],
        Form::Native => vec![
            format!("self.{}(value.to_string());", setter),
            "Ok(())".to_string(),
        ],
    }
}
