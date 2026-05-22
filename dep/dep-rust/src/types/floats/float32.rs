use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// A 32-bit floating point value with total ordering and hashing.
///
/// # Normalization
/// The wrapped value is normalized in the constructor so that bitwise-equal representations
/// always correspond to semantically-equal values:
/// 1. All NaN bit patterns collapse to a single canonical NaN.
/// 2. Negative zero (`-0.0`) collapses to positive zero (`+0.0`).
#[repr(transparent)]
#[derive(Copy, Clone, Default)]
pub struct Float32 {
    value: f32,
}

impl Float32 {
    //! Construction

    /// Creates a new [Float32] from the `value`. Normalizes NaN and `-0.0`.
    #[must_use]
    pub const fn new(value: f32) -> Self {
        let bits: u32 = value.to_bits();
        let normalized: u32 = if (bits & 0x7F800000) == 0x7F800000 && (bits & 0x007FFFFF) != 0 {
            // NaN: collapse to a canonical quiet NaN.
            0x7FC00000
        } else if bits == 0x80000000 {
            // -0.0: collapse to +0.0.
            0
        } else {
            bits
        };
        Self {
            value: f32::from_bits(normalized),
        }
    }
}

impl Float32 {
    //! Properties

    /// Gets the value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.value
    }
}

impl From<f32> for Float32 {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<Float32> for f32 {
    fn from(f: Float32) -> Self {
        f.value
    }
}

impl PartialEq for Float32 {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits()
    }
}

impl Eq for Float32 {}

impl PartialOrd for Float32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Float32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.total_cmp(&other.value)
    }
}

impl Hash for Float32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.to_bits().hash(state);
    }
}

impl Debug for Float32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.value, f)
    }
}

impl Display for Float32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value, f)
    }
}
