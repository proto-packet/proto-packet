use crate::rust::GenRust;
use code_gen::rust::{Reference, RustType};
use proto_packet_tree::{TypeTag, WithFieldName, WithTypeTag};

impl GenRust<'_> {
    //! Typing: Reference Expression

    /// Generates the reference expression for the `field`.
    pub fn reference_expression<F>(&self, field: &F, is_optional: bool) -> String
    where
        F: WithFieldName + WithTypeTag,
    {
        match field.type_tag() {
            TypeTag::Primitive(_) | TypeTag::Special(_) | TypeTag::Time(_) => {
                let name: String = self.field_name(field);
                self.into_expression(field, is_optional, &format!("self.{name}"))
            }
            TypeTag::Named(name) => {
                // todo -- named copy types
                let rust_name: String = self.rust_name(name.to_ref());
                let tag: RustType = RustType::from(rust_name).to_ref(Reference::default());
                if is_optional {
                    RustType::from("Option").with_generic(tag)
                } else {
                    tag
                }
                .to_string()
            }
        }
    }
}
