use serde::Deserializer;
use serde::de::{Error, Unexpected, Visitor};
use std::fmt::Formatter;

/// The JSON of the NaN.
pub(crate) const NAN: &str = "NaN";

/// The JSON of the positive infinity.
pub(crate) const INFINITY: &str = "Infinity";

/// The JSON of the negative infinity.
pub(crate) const NEGATIVE_INFINITY: &str = "-Infinity";

/// Deserializes a float from a number, or from the name of a value that isn't finite.
pub(crate) fn deserialize_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(FloatVisitor)
}

/// Visits a number, or the name of a float that isn't finite.
struct FloatVisitor;

impl Visitor<'_> for FloatVisitor {
    type Value = f64;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a number, `{}`, `{}` or `{}`",
            NAN, INFINITY, NEGATIVE_INFINITY
        )
    }

    fn visit_i64<E>(self, value: i64) -> Result<f64, E> {
        Ok(value as f64)
    }

    fn visit_u64<E>(self, value: u64) -> Result<f64, E> {
        Ok(value as f64)
    }

    fn visit_f64<E>(self, value: f64) -> Result<f64, E> {
        Ok(value)
    }

    fn visit_str<E>(self, name: &str) -> Result<f64, E>
    where
        E: Error,
    {
        match name {
            NAN => Ok(f64::NAN),
            INFINITY => Ok(f64::INFINITY),
            NEGATIVE_INFINITY => Ok(f64::NEG_INFINITY),
            _ => Err(E::invalid_value(Unexpected::Str(name), &self)),
        }
    }
}

/// Implements a float type that has one NaN.
///
/// The type holds the `$float` in a `value` field, with the `$bits` as its bits & the `$nan` as the
/// bits of its one NaN. It serializes with the `$serialize` function, & deserializes from an `f64`
/// with the `$narrow` function.
macro_rules! impl_float {
    ($name:ident, $float:ty, $bits:ty, $nan:expr, $serialize:ident, $narrow:expr) => {
        impl $name {
            //! Constants

            /// The bits of the one NaN.
            pub const NAN_BITS: $bits = $nan;

            /// The one NaN.
            pub const NAN: Self = Self {
                value: <$float>::from_bits($nan),
            };
        }

        impl $name {
            //! Validation

            /// Checks if the `bits` are a value. Any NaN but the one NaN is not.
            #[must_use]
            pub const fn is_valid_bits(bits: $bits) -> bool {
                !<$float>::from_bits(bits).is_nan() || bits == Self::NAN_BITS
            }
        }

        impl $name {
            //! Construction

            /// Creates the float.
            ///
            /// # Note
            /// - Any NaN becomes the one NaN.
            pub const fn new(value: $float) -> Self {
                if value.is_nan() {
                    Self::NAN
                } else {
                    Self { value }
                }
            }

            /// Creates the float from its `bits`.
            ///
            /// Returns `None` if the `bits` are a NaN that isn't the one NaN.
            #[must_use]
            pub const fn from_bits(bits: $bits) -> Option<Self> {
                if Self::is_valid_bits(bits) {
                    Some(unsafe { Self::from_bits_unchecked(bits) })
                } else {
                    None
                }
            }

            /// Creates the float from its `bits`, without checking them.
            ///
            /// # Safety
            /// The `bits` must not be a NaN that isn't the one NaN.
            pub const unsafe fn from_bits_unchecked(bits: $bits) -> Self {
                Self {
                    value: <$float>::from_bits(bits),
                }
            }
        }

        impl From<$float> for $name {
            fn from(value: $float) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for $float {
            fn from(value: $name) -> Self {
                value.value
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.value.total_cmp(&other.value)
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Eq for $name {}

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.to_bits() == other.to_bits()
            }
        }

        impl std::hash::Hash for $name {
            fn hash<H>(&self, state: &mut H)
            where
                H: std::hash::Hasher,
            {
                self.to_bits().hash(state);
            }
        }

        impl $name {
            //! Properties

            /// Gets the value.
            #[must_use]
            pub const fn get(self) -> $float {
                self.value
            }

            /// Gets the bits.
            #[must_use]
            pub const fn to_bits(self) -> $bits {
                self.value.to_bits()
            }

            /// Gets the little-endian bytes.
            #[must_use]
            pub const fn to_le_bytes(self) -> [u8; size_of::<$float>()] {
                self.value.to_le_bytes()
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self.value, f)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.value, f)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                if self.value.is_finite() {
                    serializer.$serialize(self.value)
                } else if self.value.is_nan() {
                    serializer.serialize_str($crate::types::impl_float::NAN)
                } else if self.value > 0.0 {
                    serializer.serialize_str($crate::types::impl_float::INFINITY)
                } else {
                    serializer.serialize_str($crate::types::impl_float::NEGATIVE_INFINITY)
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value: f64 = $crate::types::impl_float::deserialize_float(deserializer)?;
                let narrow: fn(f64) -> Option<$float> = $narrow;
                narrow(value).map(Self::new).ok_or_else(|| {
                    <D::Error as serde::de::Error>::custom("the number is out of range")
                })
            }
        }
    };
}

pub(crate) use impl_float;
