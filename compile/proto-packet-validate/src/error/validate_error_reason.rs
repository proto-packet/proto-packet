use proto_packet_tree::TreeError;
use std::fmt::{Display, Formatter};

/// The reason for a validation error.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValidateErrorReason {
    /// An invalid name, with the message of the rule it breaks.
    InvalidName { message: &'static str },

    /// A boxed type that is not a named type. (ex: `box u8`)
    BoxedNonNamedType,

    /// An error constructing the tree. (ex: a duplicate field name)
    Tree(TreeError),
}

impl ValidateErrorReason {
    //! Properties

    /// Gets the error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidName { .. } => "V_INVALID_NAME",
            Self::BoxedNonNamedType => "V_BOXED_NON_NAMED_TYPE",
            Self::Tree(error) => match error {
                TreeError::DuplicateFieldName { .. } => "V_DUPLICATE_FIELD_NAME",
                TreeError::DuplicateCaseName { .. } => "V_DUPLICATE_CASE_NAME",
                TreeError::DuplicateCallName { .. } => "V_DUPLICATE_CALL_NAME",
                TreeError::DuplicateTagNumber { .. } => "V_DUPLICATE_TAG_NUMBER",
                TreeError::DuplicateImportName { .. } => "V_DUPLICATE_IMPORT_NAME",
                TreeError::DuplicateTypeDecName { .. } => "V_DUPLICATE_TYPE_NAME",
                TreeError::ImportNameConflictsWithTypeDec { .. } => "V_IMPORT_NAME_CONFLICT",
                TreeError::DuplicateModPath { .. } => "V_DUPLICATE_MOD_PATH",
            },
        }
    }
}

impl Display for ValidateErrorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName { message } => f.write_str(message),
            Self::BoxedNonNamedType => f.write_str("only named types can be boxed"),
            Self::Tree(error) => Display::fmt(error, f),
        }
    }
}
