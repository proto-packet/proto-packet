use std::fmt::{Display, Formatter};

/// The reason for a validation error.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValidateErrorReason {
    /// An invalid name, with the message of the rule it breaks.
    InvalidName { message: &'static str },

    /// A boxed type that is not a named type. (ex: `box u8`)
    BoxedNonNamedType,
}

impl ValidateErrorReason {
    //! Properties

    /// Gets the error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidName { .. } => "V_INVALID_NAME",
            Self::BoxedNonNamedType => "V_BOXED_NON_NAMED_TYPE",
        }
    }
}

impl Display for ValidateErrorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName { message } => f.write_str(message),
            Self::BoxedNonNamedType => f.write_str("only named types can be boxed"),
        }
    }
}
