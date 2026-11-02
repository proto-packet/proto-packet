use crate::{CallName, CaseName, FieldName, TypeName};
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

    /// A case with the same name already exists. (enums & variants)
    DuplicateCaseName {
        type_name: TypeName,
        case_name: CaseName,
    },

    /// A call with the same name already exists. (services)
    DuplicateCallName {
        type_name: TypeName,
        call_name: CallName,
    },

    /// A field or case with the same tag number already exists. (messages, enums & variants)
    DuplicateTagNumber {
        type_name: TypeName,
        tag_number: TagNumber,
    },

    /// An import with the same effective name already exists. (schema files)
    DuplicateImportName { type_name: TypeName },

    /// A type declaration with the same name already exists. (schema files)
    DuplicateTypeDecName { type_name: TypeName },

    /// An import effective name matches a type declaration name. (schema files)
    ImportNameConflictsWithTypeDec { type_name: TypeName },
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
            Self::DuplicateCallName {
                type_name,
                call_name,
            } => write!(f, "duplicate call name `{}` in `{}`", call_name, type_name),
            Self::DuplicateTagNumber {
                type_name,
                tag_number,
            } => write!(
                f,
                "duplicate tag number `{}` in `{}`",
                tag_number, type_name
            ),
            Self::DuplicateImportName { type_name } => {
                write!(f, "duplicate import name `{}`", type_name)
            }
            Self::DuplicateTypeDecName { type_name } => {
                write!(f, "duplicate type name `{}`", type_name)
            }
            Self::ImportNameConflictsWithTypeDec { type_name } => {
                write!(
                    f,
                    "import name `{}` conflicts with a type declaration",
                    type_name
                )
            }
        }
    }
}

impl std::error::Error for TreeError {}
