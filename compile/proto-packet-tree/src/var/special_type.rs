use crate::TypeTag;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// A special type.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum SpecialType {
    /// A 16-byte identifier.
    Uuid,
}

impl SpecialType {
    //! Conversions

    /// Converts the special type to a type tag.
    #[must_use]
    pub fn to_type_tag(self) -> TypeTag {
        TypeTag::from(self)
    }
}

impl AsRef<str> for SpecialType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Uuid => "uuid",
        }
    }
}

impl Debug for SpecialType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl Display for SpecialType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for SpecialType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "uuid" => Self::Uuid,
            _ => return Err(()),
        })
    }
}
