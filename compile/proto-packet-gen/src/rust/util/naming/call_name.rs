use crate::rust::GenRust;
use proto_packet_tree::WithCallName;

impl GenRust<'_> {
    //! Naming: Call Name

    /// Gets the call name for the `call`.
    pub fn call_name(self, call: &impl WithCallName) -> String {
        call.call_name().to_string()
    }
}
