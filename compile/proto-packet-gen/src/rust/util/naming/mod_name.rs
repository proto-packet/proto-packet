use crate::rust::gen_rust::GenRust;
use proto_packet_tree::{ModName, WithTypeName};

impl GenRust<'_> {
    //! Naming: Mod Name

    /// Gets the mod name for the `element`.
    pub fn mod_name(self, element: &impl WithTypeName) -> ModName {
        let mod_name: String = self.pascal_to_snake_case(element.type_name().as_ref());
        unsafe { ModName::new_unchecked(mod_name) }
    }

    /// Gets the mod name for the `element`.
    pub fn mod_name_owned(self, element: &impl WithTypeName) -> ModName {
        self.mod_name(element)
    }
}
