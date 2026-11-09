use std::fmt::{Display, Formatter};

/// An error decoding an encoded value.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DecodeError {
    /// The bytes are shorter than the value needs.
    TooShort { expected: usize, actual: usize },

    /// The bytes are longer than the value needs.
    TooLong { expected: usize, actual: usize },

    /// An end table decreases, or goes past the end of its tail.
    InvalidEndTable,

    /// A message's tags aren't strictly increasing, or one isn't a valid tag number.
    InvalidTags,

    /// A string is not valid UTF-8.
    InvalidUtf8,

    /// An enum value or a variant's tag is not a valid tag number.
    InvalidTagNumber { value: u16 },

    /// A padding byte or an unused bit is not 0.
    InvalidPadding,

    /// An absent field is not zeros, or its entry is not empty.
    InvalidAbsentField,

    /// A `bool` byte is not 0 or 1.
    InvalidBool { value: u8 },

    /// A float is a NaN that is not the one NaN.
    InvalidFloat,
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort { expected, actual } => {
                write!(f, "expected at least {} bytes, got {}", expected, actual)
            }
            Self::TooLong { expected, actual } => {
                write!(f, "expected {} bytes, got {}", expected, actual)
            }
            Self::InvalidEndTable => f.write_str("invalid end table"),
            Self::InvalidTags => f.write_str("invalid tags"),
            Self::InvalidUtf8 => f.write_str("invalid UTF-8"),
            Self::InvalidTagNumber { value } => write!(f, "invalid tag number: {}", value),
            Self::InvalidPadding => f.write_str("invalid padding"),
            Self::InvalidAbsentField => f.write_str("invalid absent field"),
            Self::InvalidBool { value } => write!(f, "invalid bool: {}", value),
            Self::InvalidFloat => f.write_str("invalid float"),
        }
    }
}

impl std::error::Error for DecodeError {}
