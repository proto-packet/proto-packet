use crate::rust::GenRust;
use code_gen::Block;
use proto_packet_tree::{Enum, ModName};

impl GenRust<'_> {
    //! Gen Enum

    pub(in crate::rust) fn gen_enum(self, e: &Enum) -> Vec<(ModName, Block)> {
        vec![(self.mod_name(e), self.gen_enum_owned(e))]
    }
}
