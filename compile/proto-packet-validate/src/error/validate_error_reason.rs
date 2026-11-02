use std::fmt::{Display, Formatter};

/// The reason for a validation error.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValidateErrorReason {
    /// An invalid name, with the message of the rule it breaks.
    InvalidName { message: &'static str },
}

impl ValidateErrorReason {
    //! Properties

    /// Gets the error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidName { .. } => "V_INVALID_NAME",
        }
    }
}

impl Display for ValidateErrorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName { message } => f.write_str(message),
        }
    }
}
