use crate::types::impl_float::impl_float;

/// A 64-bit float with one NaN.
///
/// # Note
/// - Equal values have equal bits, so it has `Eq`, `Ord` & `Hash`.
/// - `-0.0` is less than `0.0`, & they are not equal.
/// - Every NaN is the same value, greater than every other value.
#[must_use]
#[derive(Copy, Clone, Default)]
pub struct Float64 {
    value: f64,
}

impl_float!(
    Float64,
    f64,
    u64,
    0x7FF8_0000_0000_0000,
    serialize_f64,
    Some
);
