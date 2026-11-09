use crate::types::impl_float::impl_float;

/// A 32-bit float with one NaN.
///
/// # Note
/// - Equal values have equal bits, so it has `Eq`, `Ord` & `Hash`.
/// - `-0.0` is less than `0.0`, & they are not equal.
/// - Every NaN is the same value, greater than every other value.
#[derive(Copy, Clone, Default)]
pub struct Float32 {
    value: f32,
}

impl_float!(Float32, f32, u32, 0x7FC0_0000);
