use crate::rust::GenRust;
use code_gen::rust::{RustPrimitive, RustType};
use proto_packet_tree::{PrimitiveType, TypeTag, WithTypeTag};

impl GenRust<'_> {
    //! Typing: Field Type

    pub fn field_type(self, field: &impl WithTypeTag, is_optional: bool) -> RustType {
        let base: RustType = self.field_type_direct(field);
        if is_optional { base.to_option() } else { base }
    }

    fn field_type_direct(self, field: &impl WithTypeTag) -> RustType {
        match field.type_tag() {
            TypeTag::Primitive(primitive) => match primitive {
                PrimitiveType::Boolean => RustPrimitive::Boolean.to_rust_type(),
                PrimitiveType::UnsignedInt8 => RustPrimitive::UnsignedInt8.to_rust_type(),
                PrimitiveType::UnsignedInt16 => RustPrimitive::UnsignedInt16.to_rust_type(),
                PrimitiveType::UnsignedInt32 => RustPrimitive::UnsignedInt32.to_rust_type(),
                PrimitiveType::UnsignedInt64 => RustPrimitive::UnsignedInt64.to_rust_type(),
                PrimitiveType::UnsignedInt128 => RustPrimitive::UnsignedInt128.to_rust_type(),
                PrimitiveType::SignedInt8 => RustPrimitive::SignedInt8.to_rust_type(),
                PrimitiveType::SignedInt16 => RustPrimitive::SignedInt16.to_rust_type(),
                PrimitiveType::SignedInt32 => RustPrimitive::SignedInt32.to_rust_type(),
                PrimitiveType::SignedInt64 => RustPrimitive::SignedInt64.to_rust_type(),
                PrimitiveType::SignedInt128 => RustPrimitive::SignedInt128.to_rust_type(),
            },
            TypeTag::Named(name) => RustType::Named(self.rust_name(name.to_ref())),
        }
    }
}
