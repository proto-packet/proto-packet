use crate::TypeTag;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// A time type.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum TimeType {
    /// A point in time represented as Unix epoch milliseconds in a 64-bit signed integer.
    Timestamp,

    /// A calendar date (year, month, day) without a time or timezone.
    Date,
}

impl TimeType {
    //! Conversions

    /// Converts the time type to a type tag.
    #[must_use]
    pub fn to_type_tag(self) -> TypeTag {
        TypeTag::from(self)
    }
}

impl AsRef<str> for TimeType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Timestamp => "timestamp",
            Self::Date => "date",
        }
    }
}

impl Debug for TimeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl Display for TimeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for TimeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "timestamp" => Self::Timestamp,
            "date" => Self::Date,
            _ => return Err(()),
        })
    }
}
