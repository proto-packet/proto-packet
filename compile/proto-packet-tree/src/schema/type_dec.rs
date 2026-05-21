use crate::{Enum, Message, Struct, TypeNameRef, Variant, WithTypeName};

/// A type declaration.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TypeDec {
    Struct(Struct),
    Message(Message),
    Variant(Variant),
    Enum(Enum),
}

impl From<Struct> for TypeDec {
    fn from(structure: Struct) -> Self {
        Self::Struct(structure)
    }
}

impl From<Message> for TypeDec {
    fn from(message: Message) -> Self {
        Self::Message(message)
    }
}

impl From<Variant> for TypeDec {
    fn from(variant: Variant) -> Self {
        Self::Variant(variant)
    }
}

impl From<Enum> for TypeDec {
    fn from(e: Enum) -> Self {
        Self::Enum(e)
    }
}

impl WithTypeName for TypeDec {
    fn type_name(&self) -> TypeNameRef<'_> {
        match self {
            Self::Struct(s) => s.type_name(),
            Self::Message(m) => m.type_name(),
            Self::Variant(v) => v.type_name(),
            Self::Enum(e) => e.type_name(),
        }
    }
}
