/// Implements a time type that holds one integer.
///
/// The type holds the `$int` in the `$field`, which is also the name of its getter.
macro_rules! impl_time {
    ($name:ident, $field:ident, $int:ty) => {
        impl $name {
            //! Construction

            #[doc = concat!("Creates the [", stringify!($name), "].")]
            #[must_use]
            pub const fn new($field: $int) -> Self {
                Self { $field }
            }
        }

        impl From<$int> for $name {
            fn from($field: $int) -> Self {
                Self::new($field)
            }
        }

        impl From<$name> for $int {
            fn from(value: $name) -> Self {
                value.$field
            }
        }

        impl $name {
            //! Properties

            #[doc = concat!("Gets the ", stringify!($field), ".")]
            #[must_use]
            pub const fn $field(self) -> $int {
                self.$field
            }

            /// Gets the little-endian bytes.
            #[must_use]
            pub const fn to_le_bytes(self) -> [u8; size_of::<$int>()] {
                self.$field.to_le_bytes()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.$field, f)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serde::Serialize::serialize(&self.$field, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <$int as serde::Deserialize>::deserialize(deserializer).map(Self::new)
            }
        }
    };
}

/// Implements the conversions of a time type to & from the system time.
///
/// The type holds the count of units since the epoch in the `$field`, with the `$nanos` in a unit.
macro_rules! impl_system_time {
    ($name:ident, $field:ident, $nanos:expr) => {
        impl TryFrom<std::time::SystemTime> for $name {
            type Error = $crate::types::TimeRangeError;

            /// Converts the system time, rounded down to the unit.
            fn try_from(time: std::time::SystemTime) -> Result<Self, Self::Error> {
                let nanos: u128 = $nanos;
                let count: i128 = match time.duration_since(std::time::UNIX_EPOCH) {
                    Ok(after) => (after.as_nanos() / nanos) as i128,
                    Err(before) => -(before.duration().as_nanos().div_ceil(nanos) as i128),
                };
                i64::try_from(count)
                    .map(Self::new)
                    .map_err(|_| $crate::types::TimeRangeError::default())
            }
        }

        impl TryFrom<$name> for std::time::SystemTime {
            type Error = $crate::types::TimeRangeError;

            fn try_from(time: $name) -> Result<Self, Self::Error> {
                let nanos: u128 = time.$field.unsigned_abs() as u128 * $nanos;
                let seconds: Option<u64> = u64::try_from(nanos / 1_000_000_000).ok();
                let duration: Option<std::time::Duration> = seconds.map(|seconds| {
                    std::time::Duration::new(seconds, (nanos % 1_000_000_000) as u32)
                });
                duration
                    .and_then(|duration| {
                        if time.$field >= 0 {
                            std::time::UNIX_EPOCH.checked_add(duration)
                        } else {
                            std::time::UNIX_EPOCH.checked_sub(duration)
                        }
                    })
                    .ok_or_else($crate::types::TimeRangeError::default)
            }
        }
    };
}

pub(crate) use impl_system_time;
pub(crate) use impl_time;
