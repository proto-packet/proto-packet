use crate::{Enum, Message, Service, Struct, TypeNameRef, Variant, WithTypeName};

/// A type declaration.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TypeDec {
    Struct(Struct),
    Message(Message),
    Enum(Enum),
    Variant(Variant),
    Service(Service),
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

impl From<Enum> for TypeDec {
    fn from(enumeration: Enum) -> Self {
        Self::Enum(enumeration)
    }
}

impl From<Variant> for TypeDec {
    fn from(variant: Variant) -> Self {
        Self::Variant(variant)
    }
}

impl From<Service> for TypeDec {
    fn from(service: Service) -> Self {
        Self::Service(service)
    }
}

impl WithTypeName for TypeDec {
    fn type_name(&self) -> TypeNameRef<'_> {
        match self {
            Self::Struct(structure) => structure.type_name(),
            Self::Message(message) => message.type_name(),
            Self::Enum(enumeration) => enumeration.type_name(),
            Self::Variant(variant) => variant.type_name(),
            Self::Service(service) => service.type_name(),
        }
    }
}
