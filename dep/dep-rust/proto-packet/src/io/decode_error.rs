use std::fmt::{Display, Formatter};

/// An error decoding the encoding of a value.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DecodeError {
    /// The bytes are not the length of the encoding.
    ///
    /// When the bytes are shorter than a struct's head, the expected length is the head's size.
    InvalidLength { expected: usize, actual: usize },

    /// A string is not valid UTF-8.
    InvalidUtf8,

    /// An enum tag number is not one of the enum's cases.
    InvalidEnumTag { tag_number: u16 },

    /// A date is outside the range of the dates. (in days since 1970-01-01)
    InvalidDate { days: i32 },
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => {
                write!(f, "expected {} bytes, got {}", expected, actual)
            }
            Self::InvalidUtf8 => f.write_str("invalid UTF-8"),
            Self::InvalidEnumTag { tag_number } => {
                write!(f, "the enum tag number {} is not a case", tag_number)
            }
            Self::InvalidDate { days } => {
                write!(f, "the date {} days from 1970-01-01 is out of range", days)
            }
        }
    }
}

impl std::error::Error for DecodeError {}
