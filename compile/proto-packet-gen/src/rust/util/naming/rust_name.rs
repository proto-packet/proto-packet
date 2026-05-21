use crate::rust::GenRust;
use proto_packet_tree::QualifiedNameRef;

impl GenRust<'_> {
    //! Naming: Rust Name

    /// Converts the `name` to a rust name.
    pub fn rust_name(self, name: QualifiedNameRef) -> String {
        let mut s: String = String::with_capacity(64);
        s.push_str("crate::");
        if let Some(mod_path) = name.mod_path() {
            for mod_name in mod_path.mod_names() {
                s.push_str(&mod_name);
                s.push_str("::");
            }
        }
        s.push_str(self.type_name(&name).as_str());
        s
    }
}
