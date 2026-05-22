use crate::rust::GenRust;
use code_gen::rust::{RustPrimitive, RustType};
use proto_packet_tree::{PrimitiveType, SpecialType, TimeType, TypeTag, WithTypeTag};

impl GenRust<'_> {
    //! Typing: Field Type

    pub fn field_type(
        self,
        field: &impl WithTypeTag,
        is_optional: bool,
        is_internal: bool,
    ) -> RustType {
        let base: RustType = self.field_type_direct(field, is_internal);
        if is_optional { base.to_option() } else { base }
    }

    fn field_type_direct(self, field: &impl WithTypeTag, is_internal: bool) -> RustType {
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
                PrimitiveType::Float32 => {
                    if is_internal {
                        RustType::from("proto_packet::types::Float32")
                    } else {
                        RustPrimitive::Float32.to_rust_type()
                    }
                }
                PrimitiveType::Float64 => {
                    if is_internal {
                        RustType::from("proto_packet::types::Float64")
                    } else {
                        RustPrimitive::Float64.to_rust_type()
                    }
                }
            },
            TypeTag::Special(special) => match special {
                SpecialType::Uuid => RustType::from("proto_packet::types::Uuid"),
                SpecialType::String => RustType::from("String"),
            },
            TypeTag::Time(time) => match time {
                TimeType::Timestamp => RustPrimitive::SignedInt64.to_rust_type(),
                TimeType::Date => RustType::from("proto_packet::types::Date"),
            },
            TypeTag::Named(name) => RustType::Named(self.rust_name(name.to_ref())),
        }
    }
}
