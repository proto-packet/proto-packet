use crate::types::time::impl_time::{impl_system_time, impl_time};

/// A time, as the nanoseconds since the epoch.
///
/// # Note
/// - The epoch is `1970-01-01 00:00:00 UTC`, & leap seconds aren't counted.
/// - It holds the years 1677 to 2262.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct Nanotime {
    nanos: i64,
}

impl_time!(Nanotime, nanos, i64);
impl_system_time!(Nanotime, nanos, 1);
