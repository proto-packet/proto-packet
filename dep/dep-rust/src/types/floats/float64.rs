use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// A 64-bit floating point value with total ordering and hashing.
///
/// # Normalization
/// The wrapped value is normalized in the constructor so that bitwise-equal representations
/// always correspond to semantically-equal values:
/// 1. All NaN bit patterns collapse to a single canonical NaN.
/// 2. Negative zero (`-0.0`) collapses to positive zero (`+0.0`).
#[repr(transparent)]
#[derive(Copy, Clone, Default)]
pub struct Float64 {
    value: f64,
}

impl Float64 {
    //! Construction

    /// Creates a new [Float64] from the `value`. Normalizes NaN and `-0.0`.
    #[must_use]
    pub const fn new(value: f64) -> Self {
        let bits: u64 = value.to_bits();
        let normalized: u64 = if (bits & 0x7FF0000000000000) == 0x7FF0000000000000
            && (bits & 0x000FFFFFFFFFFFFF) != 0
        {
            // NaN: collapse to a canonical quiet NaN.
            0x7FF8000000000000
        } else if bits == 0x8000000000000000 {
            // -0.0: collapse to +0.0.
            0
        } else {
            bits
        };
        Self {
            value: f64::from_bits(normalized),
        }
    }
}

impl Float64 {
    //! Properties

    /// Gets the value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.value
    }
}

impl From<f64> for Float64 {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl From<Float64> for f64 {
    fn from(f: Float64) -> Self {
        f.value
    }
}

impl PartialEq for Float64 {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits()
    }
}

impl Eq for Float64 {}

impl PartialOrd for Float64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Float64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.total_cmp(&other.value)
    }
}

impl Hash for Float64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.to_bits().hash(state);
    }
}

impl Debug for Float64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.value, f)
    }
}

impl Display for Float64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value, f)
    }
}
