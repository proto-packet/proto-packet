use crate::rust::naming::{ident, is_renamed, is_snake_case};
use crate::rust::typing::{Typing, option};
use code_gen::WithStatements;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, Reference, RustType, Signature, StructField, Var,
    WithAccess, WithAttributes, WithComments, WithFnGenerics, WithFunctions, WithReceiver,
    WithResult, WithVarParams,
};
use proto_packet_tree::{FieldNameRef, TypeTag};

/// A field of a generated struct or message.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Field<'a> {
    pub(crate) name: FieldNameRef<'a>,
    pub(crate) tag: &'a TypeTag,
    pub(crate) is_optional: bool,
    pub(crate) is_boxed: bool,
}

impl Field<'_> {
    //! Types

    /// Gets the value type of the field's API. (optional fields are `Option`s)
    pub(crate) fn value_type(self, typing: Typing<'_>) -> RustType {
        let value_type: RustType = typing.value_type(self.tag);
        if self.is_optional {
            option(value_type)
        } else {
            value_type
        }
    }

    /// Gets the storage type of the field. (boxed fields are `Box`es, optional fields `Option`s)
    pub(crate) fn storage_type(self, typing: Typing<'_>) -> RustType {
        let mut storage_type: RustType = typing.storage_type(self.tag);
        if self.is_boxed {
            storage_type = RustType::from("::std::boxed::Box").with_generic(storage_type);
        }
        if self.is_optional {
            option(storage_type)
        } else {
            storage_type
        }
    }

    /// Converts the value `expr` of the field to its storage type.
    pub(crate) fn storage_expr(self, typing: Typing<'_>, expr: &str) -> String {
        if self.is_boxed {
            if self.is_optional {
                format!("{}.map(::std::boxed::Box::new)", expr)
            } else {
                format!("::std::boxed::Box::new({})", expr)
            }
        } else if self.is_optional {
            typing.storage_expr_option(self.tag, expr)
        } else {
            typing.storage_expr(self.tag, expr)
        }
    }

    /// Converts the storage `expr` of the field to its value type.
    fn value_expr(self, typing: Typing<'_>, expr: &str) -> String {
        if self.is_boxed {
            if self.is_optional {
                format!("{}.map(|value| *value)", expr)
            } else {
                format!("*{}", expr)
            }
        } else if self.is_optional {
            typing.value_expr_option(self.tag, expr)
        } else {
            typing.value_expr(self.tag, expr)
        }
    }
}

impl Field<'_> {
    //! Getters

    /// Gets the return type & expression of the field's getter. (boxes are hidden)
    fn getter(self, typing: Typing<'_>) -> (RustType, String) {
        let field: String = format!("self.{}", ident(&self.name));
        if !self.is_boxed {
            return typing.getter(self.tag, &ident(&self.name), self.is_optional);
        }
        let (result, expr): (RustType, String) = if typing.is_copy(self.tag) {
            let expr: String = if self.is_optional {
                format!("{}.as_deref().copied()", field)
            } else {
                format!("*{}", field)
            };
            (typing.value_type(self.tag), expr)
        } else {
            let expr: String = if self.is_optional {
                format!("{}.as_deref()", field)
            } else {
                format!("&{}", field)
            };
            (
                typing.value_type(self.tag).to_ref(Reference::default()),
                expr,
            )
        };
        if self.is_optional {
            (option(result), expr)
        } else {
            (result, expr)
        }
    }
}

impl Field<'_> {
    //! Declaration

    /// Generates the declaration of the field.
    ///
    /// A renamed field keeps its schema name in serde. (ex: `self_` is `self`)
    pub(crate) fn gen_dec(self, typing: Typing<'_>) -> StructField {
        let var: Var = Var::from((ident(&self.name), self.storage_type(typing)));
        let mut field: StructField = StructField::from(var);
        if is_renamed(&self.name) {
            field.add_attribute(format!("serde(rename = \"{}\")", self.name));
        }
        if self.is_optional {
            field.add_attribute("serde(skip_serializing_if = \"::std::option::Option::is_none\")");
        }
        field
    }
}

impl Field<'_> {
    //! Accessors

    /// Checks if the field name is not snake case for Rust. (ex: `myField`)
    pub(crate) fn is_non_snake_case(self) -> bool {
        !is_snake_case(&self.name)
    }

    /// Generates the impl block with the getter, setter & builder of the field.
    pub(crate) fn gen_impl(self, typing: Typing<'_>, type_name: &str) -> ImplBlock {
        let mut functions: [Function; 3] = [
            self.gen_getter(typing),
            self.gen_setter(typing),
            self.gen_builder(typing),
        ];
        if self.is_non_snake_case() {
            for function in &mut functions {
                function.add_attribute("allow(non_snake_case)");
            }
        }
        let [getter, setter, builder] = functions;
        ImplBlock::from(type_name)
            .with_comment(format!(" Field: `{}`", self.name))
            .with_function(getter)
            .with_function(setter)
            .with_function(builder)
    }

    /// Generates the getter of the field.
    fn gen_getter(self, typing: Typing<'_>) -> Function {
        let (result, expr): (RustType, String) = self.getter(typing);
        let signature: Signature = Signature::from(ident(&self.name))
            .with_receiver(Receiver::Borrowed)
            .with_result(result);
        Function::from(signature)
            .with_comment(format!(" Gets the field: `{}`.", self.name))
            .with_attribute("must_use")
            .with_access(Access::Public)
            .with_literal(expr)
    }

    /// Generates the setter of the field. (it returns the previous value)
    fn gen_setter(self, typing: Typing<'_>) -> Function {
        let field: String = ident(&self.name);
        let value_type: RustType = self.value_type(typing);
        let mut signature: Signature = Signature::from(format!("set_{}", self.name))
            .with_receiver(Receiver::BorrowedMut)
            .with_result(value_type.clone());
        if self.is_optional {
            signature.add_generic(Var::from(("F", into(value_type.clone()))));
            signature.add_param(Var::from((field.clone(), "F")));
        } else {
            signature.add_param(Var::from((field.clone(), value_type.clone())));
        }
        let mut function: Function = Function::from(signature)
            .with_comment(format!(
                " Sets the field: `{}`. Returns the previous value.",
                self.name
            ))
            .with_access(Access::Public);
        if self.is_optional {
            function.add_semi(format!("let {}: {} = {}.into()", field, value_type, field));
        }
        let replace: String = format!(
            "::std::mem::replace(&mut self.{}, {})",
            field,
            self.storage_expr(typing, &field)
        );
        function.with_literal(self.value_expr(typing, &replace))
    }

    /// Generates the builder of the field. (it sets the field & returns itself)
    fn gen_builder(self, typing: Typing<'_>) -> Function {
        let field: String = ident(&self.name);
        let mut signature: Signature = Signature::from(format!("with_{}", self.name))
            .with_receiver(Receiver::OwnedMut)
            .with_result("Self");
        if self.is_optional {
            signature.add_generic(Var::from(("F", into(self.value_type(typing)))));
            signature.add_param(Var::from((field.clone(), "F")));
        } else {
            signature.add_param(Var::from((field.clone(), self.value_type(typing))));
        }
        Function::from(signature)
            .with_comment(format!(
                " Sets the field: `{}`. (builder pattern)",
                self.name
            ))
            .with_attribute("must_use")
            .with_access(Access::Public)
            .with_semi(format!("self.set_{}({})", self.name, field))
            .with_literal("self")
    }
}

/// Gets the `Into` bound of the `rust_type`.
fn into(rust_type: RustType) -> RustType {
    RustType::from("::std::convert::Into").with_generic(rust_type)
}
