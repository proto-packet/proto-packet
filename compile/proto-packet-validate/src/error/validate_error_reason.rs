use proto_packet_tree::TreeError;
use std::fmt::{Display, Formatter};

/// The reason for a validation error.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValidateErrorReason {
    /// An invalid name, with the message of the rule it breaks.
    InvalidName { message: &'static str },

    /// An error constructing the tree. (ex: a duplicate field name)
    Tree(TreeError),
}

impl Display for ValidateErrorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName { message } => f.write_str(message),
            Self::Tree(error) => Display::fmt(error, f),
        }
    }
}
