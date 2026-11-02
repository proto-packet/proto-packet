use crate::rust::naming::{ident, is_renamed, is_snake_case};
use crate::rust::typing::{Typing, option};
use code_gen::rust::{RustType, StructField, Var, WithAttributes};
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
        let value_type: RustType = typing.rust_type(self.tag);
        if self.is_optional {
            option(value_type)
        } else {
            value_type
        }
    }

    /// Gets the storage type of the field. (boxed fields are `Box`es, optional fields `Option`s)
    pub(crate) fn storage_type(self, typing: Typing<'_>) -> RustType {
        let mut storage_type: RustType = typing.rust_type(self.tag);
        if self.is_boxed {
            storage_type = RustType::from("::std::boxed::Box").with_generic(storage_type);
        }
        if self.is_optional {
            option(storage_type)
        } else {
            storage_type
        }
    }

    /// Converts the value `expr` of the field to its storage type. (it boxes boxed fields)
    pub(crate) fn storage_expr(self, expr: &str) -> String {
        if !self.is_boxed {
            expr.to_string()
        } else if self.is_optional {
            format!("{}.map(::std::boxed::Box::new)", expr)
        } else {
            format!("::std::boxed::Box::new({})", expr)
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
}
