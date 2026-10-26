use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Defines a float type with total ordering & hashing.
macro_rules! float_type {
    ($(#[$meta:meta])* $name:ident, $float:ty) => {
        $(#[$meta])*
        #[derive(Copy, Clone, Default)]
        pub struct $name {
            value: $float,
        }

        impl $name {
            //! Construction

            /// Creates a new float from the `value`, normalizing NaN & negative zero.
            pub const fn new(value: $float) -> Self {
                let value: $float = if value.is_nan() {
                    <$float>::NAN
                } else if value == 0.0 {
                    0.0
                } else {
                    value
                };
                Self { value }
            }
        }

        impl $name {
            //! Properties

            /// Gets the value.
            pub const fn value(self) -> $float {
                self.value
            }
        }

        impl From<$float> for $name {
            fn from(value: $float) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for $float {
            fn from(float: $name) -> Self {
                float.value
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.value.to_bits() == other.value.to_bits()
            }
        }

        impl Eq for $name {}

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                self.value.total_cmp(&other.value)
            }
        }

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.value.to_bits().hash(state);
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Debug::fmt(&self.value, f)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.value, f)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.value.serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                <$float>::deserialize(deserializer).map(Self::new)
            }
        }
    };
}

float_type!(
    /// A 32-bit float with total ordering & hashing.
    ///
    /// # Normalization
    /// The value is normalized on creation & deserialization, so equal values have equal bits:
    /// 1. Every NaN becomes the canonical NaN.
    /// 2. Negative zero becomes positive zero.
    Float32,
    f32
);

float_type!(
    /// A 64-bit float with total ordering & hashing.
    ///
    /// # Normalization
    /// The value is normalized on creation & deserialization, so equal values have equal bits:
    /// 1. Every NaN becomes the canonical NaN.
    /// 2. Negative zero becomes positive zero.
    Float64,
    f64
);
