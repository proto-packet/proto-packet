use crate::types::time::impl_time::{impl_system_time, impl_time, impl_time_text};

/// A time, as the milliseconds since the epoch.
///
/// # Note
/// - The epoch is `1970-01-01 00:00:00 UTC`, & leap seconds aren't counted.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct Timestamp {
    millis: i64,
}

impl_time!(Timestamp, millis, i64);
impl_time_text!(Timestamp, millis, i64);
impl_system_time!(Timestamp, millis, 1_000_000);
