use std::fmt::{Display, Formatter};

/// An error decoding an encoded value.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DecodeError {
    /// The bytes are shorter than the value's fixed block.
    TooShort { expected: usize, actual: usize },

    /// An end table decreases, or goes past the end of its tail.
    InvalidEndTable,

    /// A message's tags aren't strictly increasing, or one isn't a valid tag number.
    InvalidTags,

    /// A string is not valid UTF-8.
    InvalidUtf8,

    /// An enum value is not a valid tag number.
    InvalidTagNumber { value: u16 },
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort { expected, actual } => {
                write!(f, "expected at least {} bytes, got {}", expected, actual)
            }
            Self::InvalidEndTable => f.write_str("invalid end table"),
            Self::InvalidTags => f.write_str("invalid tags"),
            Self::InvalidUtf8 => f.write_str("invalid UTF-8"),
            Self::InvalidTagNumber { value } => write!(f, "invalid tag number: {}", value),
        }
    }
}

impl std::error::Error for DecodeError {}
