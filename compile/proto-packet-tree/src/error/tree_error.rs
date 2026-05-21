use crate::{CaseName, FieldName, ModPath, TypeName};
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

    /// A field or case with the same tag number already exists. (messages & variants)
    DuplicateTagNumber {
        type_name: TypeName,
        tag_number: TagNumber,
    },

    /// A type declaration with the same name already exists. (schema files)
    DuplicateTypeDecName { type_name: TypeName },

    /// An import with the same effective name already exists. (schema files)
    DuplicateImportName { type_name: TypeName },

    /// The effective name of an import collides with a type declaration name. (schema files)
    ImportNameConflictsWithTypeDec { type_name: TypeName },

    /// A schema file with the same mod path already exists. (projects)
    DuplicateModPathName { mod_path: ModPath },
}

impl Display for TreeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for TreeError {}
