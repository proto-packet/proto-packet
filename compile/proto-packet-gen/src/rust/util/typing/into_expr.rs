use crate::rust::GenRust;
use proto_packet_tree::{PrimitiveType, TypeTag, WithTypeTag};

impl GenRust<'_> {
    //! Typing: Into Expression

    /// Wraps `expr` with `.into()` (or `.map(Into::into)` for optional) if the field's type
    /// has a different storage vs. exposed representation. Returns `expr` unchanged otherwise.
    pub fn into_expression(
        self,
        field: &impl WithTypeTag,
        is_optional: bool,
        expr: &str,
    ) -> String {
        match (field.type_tag(), is_optional) {
            (TypeTag::Primitive(PrimitiveType::Float32 | PrimitiveType::Float64), true) => {
                format!("{expr}.map(Into::into)")
            }
            (TypeTag::Primitive(PrimitiveType::Float32 | PrimitiveType::Float64), false) => {
                format!("{expr}.into()")
            }
            _ => expr.to_string(),
        }
    }

    /// Like [`Self::into_expression`] but uses const-fn constructors (e.g. `Float32::new`)
    /// so the result can appear in a `const fn` body.
    pub fn into_expression_const(
        self,
        field: &impl WithTypeTag,
        is_optional: bool,
        expr: &str,
    ) -> String {
        match (field.type_tag(), is_optional) {
            (TypeTag::Primitive(PrimitiveType::Float32), true) => {
                format!("{expr}.map(proto_packet::types::Float32::new)")
            }
            (TypeTag::Primitive(PrimitiveType::Float32), false) => {
                format!("proto_packet::types::Float32::new({expr})")
            }
            (TypeTag::Primitive(PrimitiveType::Float64), true) => {
                format!("{expr}.map(proto_packet::types::Float64::new)")
            }
            (TypeTag::Primitive(PrimitiveType::Float64), false) => {
                format!("proto_packet::types::Float64::new({expr})")
            }
            _ => expr.to_string(),
        }
    }
}
