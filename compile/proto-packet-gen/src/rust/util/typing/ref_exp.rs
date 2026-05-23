use crate::rust::GenRust;
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
            TypeTag::Slice { .. } => {
                let name: String = self.field_name(field);
                format!("self.{name}")
            }
            TypeTag::Named(_) => {
                let name: String = self.field_name(field);
                format!("self.{name}")
            }
        }
    }
}
