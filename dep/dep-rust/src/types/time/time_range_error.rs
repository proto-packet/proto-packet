use std::fmt::{Display, Formatter};

/// An error from a time that the type it converts to can't hold.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct TimeRangeError {}

impl Display for TimeRangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("the time is out of range")
    }
}

impl std::error::Error for TimeRangeError {}
