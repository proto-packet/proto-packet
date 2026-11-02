use crate::{CaseName, FieldName, TypeName};
use proto_packet::io::TagNumber;
use std::fmt::{Display, Formatter};

/// An error constructing a semantic schema tree.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TreeError {
    /// A field with the same name already exists. (structs & messages)
    DuplicateFieldName {
        type_name: TypeName,
        field_name: FieldName,
    },

    /// A case with the same name already exists. (enums)
    DuplicateCaseName {
        type_name: TypeName,
        case_name: CaseName,
    },

    /// A field or case with the same tag number already exists. (messages & enums)
    DuplicateTagNumber {
        type_name: TypeName,
        tag_number: TagNumber,
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
            Self::DuplicateCaseName {
                type_name,
                case_name,
            } => write!(f, "duplicate case name `{}` in `{}`", case_name, type_name),
            Self::DuplicateTagNumber {
                type_name,
                tag_number,
            } => write!(
                f,
                "duplicate tag number `{}` in `{}`",
                tag_number, type_name
            ),
        }
    }
}

impl std::error::Error for TreeError {}
