use crate::{LinkError, NameResolver};
use proto_packet_tree::{TypeDec, TypeTag};

/// Responsible for linking types.
#[derive(Copy, Clone, Debug)]
pub struct TypeLinker<'a> {
    resolver: NameResolver<'a>,
}

impl<'a> From<NameResolver<'a>> for TypeLinker<'a> {
    fn from(resolver: NameResolver<'a>) -> Self {
        Self { resolver }
    }
}

impl TypeLinker<'_> {
    //! Link

    /// Links the `type_dec` by resolving all named type references.
    pub fn link(self, type_dec: &TypeDec) -> Result<TypeDec, LinkError> {
        Ok(match type_dec {
            TypeDec::Struct(s) => TypeDec::Struct(self.link_struct(s)?),
            TypeDec::Message(m) => TypeDec::Message(self.link_message(m)?),
            TypeDec::Variant(v) => TypeDec::Variant(self.link_variant(v)?),
            TypeDec::Enum(e) => TypeDec::Enum(e.clone()),
        })
    }
}

impl TypeLinker<'_> {
    //! Link: Type Tag

    /// Links the type `tag`.
    pub(in crate::type_linker) fn link_type_tag(self, tag: &TypeTag) -> Result<TypeTag, LinkError> {
        match tag {
            TypeTag::Primitive(primitive) => Ok(TypeTag::Primitive(*primitive)),
            TypeTag::Special(special) => Ok(TypeTag::Special(*special)),
            TypeTag::Named(name) => Ok(TypeTag::Named(self.resolver.resolve(name.to_ref())?)),
        }
    }
}
