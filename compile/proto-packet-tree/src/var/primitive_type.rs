use std::fmt::{Display, Formatter};

/// A primitive type.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PrimitiveType {
    /// A true or false value.
    Boolean,

    /// An unsigned 8-bit integer.
    UnsignedInt8,

    /// An unsigned 16-bit integer.
    UnsignedInt16,

    /// An unsigned 32-bit integer.
    UnsignedInt32,

    /// An unsigned 64-bit integer.
    UnsignedInt64,

    /// An unsigned 128-bit integer.
    UnsignedInt128,

    /// A signed 8-bit integer.
    SignedInt8,

    /// A signed 16-bit integer.
    SignedInt16,

    /// A signed 32-bit integer.
    SignedInt32,

    /// A signed 64-bit integer.
    SignedInt64,

    /// A signed 128-bit integer.
    SignedInt128,

    /// A 32-bit IEEE 754 floating point number.
    Float32,

    /// A 64-bit IEEE 754 floating point number.
    Float64,
}

impl PrimitiveType {
    //! Names

    /// The primitive types.
    const ALL: [Self; 13] = [
        Self::Boolean,
        Self::UnsignedInt8,
        Self::UnsignedInt16,
        Self::UnsignedInt32,
        Self::UnsignedInt64,
        Self::UnsignedInt128,
        Self::SignedInt8,
        Self::SignedInt16,
        Self::SignedInt32,
        Self::SignedInt64,
        Self::SignedInt128,
        Self::Float32,
        Self::Float64,
    ];

    /// Gets the name. (ex: `u8`)
    pub const fn name(self) -> &'static str {
        match self {
            Self::Boolean => "bool",
            Self::UnsignedInt8 => "u8",
            Self::UnsignedInt16 => "u16",
            Self::UnsignedInt32 => "u32",
            Self::UnsignedInt64 => "u64",
            Self::UnsignedInt128 => "u128",
            Self::SignedInt8 => "i8",
            Self::SignedInt16 => "i16",
            Self::SignedInt32 => "i32",
            Self::SignedInt64 => "i64",
            Self::SignedInt128 => "i128",
            Self::Float32 => "f32",
            Self::Float64 => "f64",
        }
    }

    /// Gets the optional primitive type with the `name`.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|primitive| primitive.name() == name)
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
