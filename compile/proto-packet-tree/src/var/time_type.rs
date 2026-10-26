use std::fmt::{Display, Formatter};

/// A time type.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TimeType {
    /// A point in time represented as Unix epoch milliseconds in a 64-bit signed integer.
    Timestamp,

    /// A calendar date (year, month, day) without a time or timezone.
    Date,
}

impl TimeType {
    //! Names

    /// The time types.
    const ALL: [Self; 2] = [Self::Timestamp, Self::Date];

    /// Gets the name. (ex: `timestamp`)
    pub const fn name(self) -> &'static str {
        match self {
            Self::Timestamp => "timestamp",
            Self::Date => "date",
        }
    }

    /// Gets the optional time type with the `name`.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|time| time.name() == name)
    }
}

impl Display for TimeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
