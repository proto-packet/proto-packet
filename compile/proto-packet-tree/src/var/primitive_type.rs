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
}

impl PrimitiveType {
    //! Names

    /// The primitive types.
    const ALL: [Self; 6] = [
        Self::Boolean,
        Self::UnsignedInt8,
        Self::UnsignedInt16,
        Self::UnsignedInt32,
        Self::UnsignedInt64,
        Self::UnsignedInt128,
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
