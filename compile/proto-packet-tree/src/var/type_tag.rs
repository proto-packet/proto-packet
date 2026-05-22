use crate::{PrimitiveType, QualifiedName, SpecialType, TimeType, WithTypeTag};
use std::fmt::{Debug, Display, Formatter};

/// A type tag.
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum TypeTag {
    /// A primitive type.
    Primitive(PrimitiveType),

    /// A special type.
    Special(SpecialType),

    /// A time type.
    Time(TimeType),

    /// A named type.
    Named(QualifiedName),

    /// A slice type.
    Slice { base: Box<TypeTag> },
}

impl From<PrimitiveType> for TypeTag {
    fn from(primitive: PrimitiveType) -> Self {
        Self::Primitive(primitive)
    }
}

impl From<SpecialType> for TypeTag {
    fn from(special: SpecialType) -> Self {
        Self::Special(special)
    }
}

impl From<TimeType> for TypeTag {
    fn from(time: TimeType) -> Self {
        Self::Time(time)
    }
}

impl From<QualifiedName> for TypeTag {
    fn from(name: QualifiedName) -> Self {
        Self::Named(name)
    }
}

impl TypeTag {
    //! Slices

    /// Converts the type tag to a slice of itself.
    pub fn to_slice(self) -> Self {
        Self::Slice {
            base: Box::new(self),
        }
    }
}

impl WithTypeTag for TypeTag {
    fn type_tag(&self) -> &TypeTag {
        self
    }
}

impl Debug for TypeTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for TypeTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(primitive) => write!(f, "{}", primitive),
            Self::Special(special) => write!(f, "{}", special),
            Self::Time(time) => write!(f, "{}", time),
            Self::Named(name) => write!(f, "{}", name),
            Self::Slice { base } => write!(f, "[]{}", base),
        }
    }
}
