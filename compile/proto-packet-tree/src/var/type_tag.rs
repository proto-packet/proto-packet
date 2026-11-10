use crate::{PrimitiveType, QualifiedName, SpecialType};
use std::fmt::{Display, Formatter};

/// A type tag.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TypeTag {
    /// A primitive type.
    Primitive(PrimitiveType),

    /// A special type.
    Special(SpecialType),

    /// A named type.
    Named(QualifiedName),

    /// A list of the element type.
    List(Box<TypeTag>),

    /// A map of the `key` type to the `value` type.
    Map {
        key: Box<TypeTag>,
        value: Box<TypeTag>,
    },

    /// An array, which is a list of exactly the `len` of the `element` type.
    Array { len: u32, element: Box<TypeTag> },

    /// An optional value of the type.
    ///
    /// # Note
    /// - It is only an element of a list or an array, or a value of a map. An optional field has
    ///   its own flag.
    Optional(Box<TypeTag>),
}

impl TypeTag {
    //! Kinds

    /// Checks if the type is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Self::Special(SpecialType::String))
    }

    /// Checks if the type is a list.
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Checks if the type is the bytes, a list of `u8`.
    pub fn is_bytes(&self) -> bool {
        matches!(self, Self::List(element) if **element == Self::Primitive(PrimitiveType::UnsignedInt8))
    }

    /// Checks if the type is a map.
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map { .. })
    }

    /// Checks if the type is a list or a map.
    pub fn is_collection(&self) -> bool {
        self.is_list() || self.is_map()
    }

    /// Checks if the type is an array.
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array { .. })
    }

    /// Checks if the type is an optional value.
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Optional(_))
    }
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

impl From<QualifiedName> for TypeTag {
    fn from(name: QualifiedName) -> Self {
        Self::Named(name)
    }
}

impl Display for TypeTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(primitive) => write!(f, "{}", primitive),
            Self::Special(special) => write!(f, "{}", special),
            Self::Named(name) => write!(f, "{}", name),
            Self::List(element) => write!(f, "[]{}", element),
            Self::Map { key, value } => write!(f, "map<{}, {}>", key, value),
            Self::Array { len, element } => write!(f, "[{}]{}", len, element),
            Self::Optional(value) => write!(f, "optional {}", value),
        }
    }
}
