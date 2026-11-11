use crate::types::TimeRangeError;
use crate::types::time::impl_time::{impl_time, impl_time_text};

/// A span of time, as nanoseconds.
///
/// # Note
/// - It can be negative, unlike the standard duration.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct Duration {
    nanos: i64,
}

impl_time!(Duration, nanos, i64);
impl_time_text!(Duration, nanos, i64);

impl TryFrom<std::time::Duration> for Duration {
    type Error = TimeRangeError;

    fn try_from(duration: std::time::Duration) -> Result<Self, TimeRangeError> {
        i64::try_from(duration.as_nanos())
            .map(Self::new)
            .map_err(|_| TimeRangeError::default())
    }
}

impl TryFrom<Duration> for std::time::Duration {
    type Error = TimeRangeError;

    fn try_from(duration: Duration) -> Result<Self, TimeRangeError> {
        u64::try_from(duration.nanos)
            .map(std::time::Duration::from_nanos)
            .map_err(|_| TimeRangeError::default())
    }
}
