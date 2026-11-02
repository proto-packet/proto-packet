use std::fmt::{Display, Formatter};

/// An error generating code.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum GenError {
    /// Two items of a module have the same generated module name. (ex: `FooBar` & `Foo_Bar`)
    ///
    /// The module name `mod` also conflicts, with the parent module's own `mod.rs` file.
    ModNameConflict { mod_path: String, mod_name: String },

    /// A field is named like a generated method of its type.
    ///
    /// (ex: the constructor `new`, or the setter `set_x` & builder `with_x` of the field `x`)
    FieldNameConflict {
        type_name: String,
        field_name: String,
    },
}

impl GenError {
    //! Properties

    /// Gets the error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ModNameConflict { .. } => "G_MOD_NAME_CONFLICT",
            Self::FieldNameConflict { .. } => "G_FIELD_NAME_CONFLICT",
        }
    }
}

impl Display for GenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModNameConflict { mod_path, mod_name } => write!(
                f,
                "the generated module name `{}` conflicts in `{}`",
                mod_name, mod_path
            ),
            Self::FieldNameConflict {
                type_name,
                field_name,
            } => write!(
                f,
                "the field name `{}` conflicts with a generated method of `{}`",
                field_name, type_name
            ),
        }
    }
}

impl std::error::Error for GenError {}
