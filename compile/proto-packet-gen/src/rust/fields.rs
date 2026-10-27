use crate::GenError;
use crate::rust::naming::{ident, is_renamed, is_snake_case};
use crate::rust::typing::{Typing, option};
use code_gen::WithStatements;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, Reference, RustType, Signature, StructField, Var,
    WithAccess, WithAttributes, WithComments, WithFnGenerics, WithFunctions, WithReceiver,
    WithResult, WithVarParams,
};
use proto_packet_tree::{FieldNameRef, TypeTag};
use std::collections::BTreeSet;

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

    /// Gets the name of the field's setter. (ex: `set_x`)
    fn setter_name(self) -> String {
        format!("set_{}", self.name)
    }

    /// Gets the name of the field's builder. (ex: `with_x`)
    fn builder_name(self) -> String {
        format!("with_{}", self.name)
    }

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
    ///
    /// The getter of a field named `new` does not return `Self`. (only messages can have one)
    fn gen_getter(self, typing: Typing<'_>) -> Function {
        let (result, expr): (RustType, String) = self.getter(typing);
        let signature: Signature = Signature::from(ident(&self.name))
            .with_receiver(Receiver::Borrowed)
            .with_result(result);
        let mut function: Function = Function::from(signature)
            .with_comment(format!(" Gets the field: `{}`.", self.name))
            .with_attribute("must_use")
            .with_access(Access::Public)
            .with_literal(expr);
        if self.name == "new" {
            function.add_attribute("allow(clippy::new_ret_no_self)");
        }
        function
    }

    /// Generates the setter of the field. (it returns the previous value)
    fn gen_setter(self, typing: Typing<'_>) -> Function {
        let field: String = ident(&self.name);
        let value_type: RustType = self.value_type(typing);
        let mut signature: Signature = Signature::from(self.setter_name())
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
        let mut signature: Signature = Signature::from(self.builder_name())
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
            .with_semi(format!("self.{}({})", self.setter_name(), field))
            .with_literal("self")
    }
}

/// Checks no field of the type with the `type_name` is named like one of its generated methods.
///
/// The methods are the type's own `methods` (ex: the constructor `new`), plus the setter & builder
/// of each field. (ex: `set_x` & `with_x` of the field `x`) The getters have the names of their
/// fields, so they cannot conflict with each other.
pub(crate) fn check_field_names(
    type_name: &str,
    fields: &[Field<'_>],
    methods: &[&str],
) -> Result<(), GenError> {
    let mut method_names: BTreeSet<String> =
        methods.iter().map(|method| method.to_string()).collect();
    for field in fields {
        method_names.insert(field.setter_name());
        method_names.insert(field.builder_name());
    }
    match fields
        .iter()
        .find(|field| method_names.contains(field.name.value()))
    {
        Some(field) => Err(GenError::FieldNameConflict {
            type_name: type_name.to_string(),
            field_name: field.name.to_string(),
        }),
        None => Ok(()),
    }
}

/// Gets the `Into` bound of the `rust_type`.
fn into(rust_type: RustType) -> RustType {
    RustType::from("::std::convert::Into").with_generic(rust_type)
}

#[cfg(test)]
mod tests {
    use crate::GenError;
    use crate::rust::fields::{Field, check_field_names};
    use proto_packet_tree::{FieldName, PrimitiveType, TypeTag};

    #[test]
    fn check_field_names_conflicts() {
        let test_cases: &[(&[&str], &[&str], Option<&str>)] = &[
            (&["x", "y"], &["new"], None),
            (&["newer", "set", "with"], &["new"], None),
            (&["new"], &["new"], Some("new")),
            (&["new"], &[], None),
            (&["x", "set_x"], &[], Some("set_x")),
            (&["with_x", "x"], &[], Some("with_x")),
            (&["type", "set_type"], &[], Some("set_type")),
        ];
        let tag: TypeTag = TypeTag::from(PrimitiveType::UnsignedInt32);
        for (field_names, methods, expected) in test_cases {
            let names: Vec<FieldName> = field_names
                .iter()
                .map(|name| FieldName::try_from(*name).unwrap())
                .collect();
            let fields: Vec<Field> = names
                .iter()
                .map(|name| Field {
                    name: name.to_ref(),
                    tag: &tag,
                    is_optional: false,
                    is_boxed: false,
                })
                .collect();
            let result: Result<(), GenError> = check_field_names("a.Foo", &fields, methods);
            let expected: Result<(), GenError> = match expected {
                Some(field_name) => Err(GenError::FieldNameConflict {
                    type_name: "a.Foo".to_string(),
                    field_name: field_name.to_string(),
                }),
                None => Ok(()),
            };
            assert_eq!(result, expected, "fields: {:?}", field_names);
        }
    }
}
