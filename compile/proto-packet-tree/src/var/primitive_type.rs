use crate::TypeTag;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// A primitive type.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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
}

impl PrimitiveType {
    //! Conversions

    /// Converts the primitive type to a type tag.
    #[must_use]
    pub fn to_type_tag(self) -> TypeTag {
        TypeTag::from(self)
    }
}

impl AsRef<str> for PrimitiveType {
    fn as_ref(&self) -> &str {
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
        }
    }
}

impl Debug for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for PrimitiveType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "bool" => Self::Boolean,
            "u8" => Self::UnsignedInt8,
            "u16" => Self::UnsignedInt16,
            "u32" => Self::UnsignedInt32,
            "u64" => Self::UnsignedInt64,
            "u128" => Self::UnsignedInt128,
            "i8" => Self::SignedInt8,
            "i16" => Self::SignedInt16,
            "i32" => Self::SignedInt32,
            "i64" => Self::SignedInt64,
            "i128" => Self::SignedInt128,
            _ => return Err(()),
        })
    }
}
