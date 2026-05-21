use crate::rust::GenRust;
use proto_packet_tree::WithCaseName;

impl GenRust<'_> {
    //! Naming: Case Name

    /// Gets the case name for the `case`.
    pub fn case_name(self, case: &impl WithCaseName) -> String {
        case.case_name().to_string()
    }
}
