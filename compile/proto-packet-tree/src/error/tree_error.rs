use crate::{FieldName, TypeName};
use std::fmt::{Display, Formatter};

/// An error constructing a semantic schema tree.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TreeError {
    /// A field with the same name already exists. (structs)
    DuplicateFieldName {
        type_name: TypeName,
        field_name: FieldName,
    },
}

impl Display for TreeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateFieldName {
                type_name,
                field_name,
            } => write!(
                f,
                "duplicate field name `{}` in `{}`",
                field_name, type_name
            ),
        }
    }
}

impl std::error::Error for TreeError {}
