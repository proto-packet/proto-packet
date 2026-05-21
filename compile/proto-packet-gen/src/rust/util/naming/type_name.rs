use crate::rust::GenRust;
use proto_packet_tree::WithTypeName;

impl GenRust<'_> {
    //! Naming: Type Name

    /// Gets the type name for the `dec`.
    pub fn type_name(self, dec: &impl WithTypeName) -> String {
        dec.type_name().to_string()
    }
}
