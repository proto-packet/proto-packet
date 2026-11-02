use std::fmt::{Display, Formatter};

/// An error generating code.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum GenError {
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
            Self::FieldNameConflict { .. } => "G_FIELD_NAME_CONFLICT",
        }
    }
}

impl Display for GenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
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
