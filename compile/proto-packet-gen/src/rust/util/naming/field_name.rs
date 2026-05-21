use crate::rust::GenRust;
use proto_packet_tree::WithFieldName;

impl GenRust<'_> {
    //! Naming: Field Name

    /// Gets the field name for the `field`.
    pub fn field_name(self, field: &impl WithFieldName) -> String {
        field.field_name().to_string()
    }
}
