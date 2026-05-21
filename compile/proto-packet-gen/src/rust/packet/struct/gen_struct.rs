use crate::rust::GenRust;
use code_gen::Block;
use proto_packet_tree::{ModName, Struct};

impl GenRust<'_> {
    //! Gen Struct

    pub(in crate::rust) fn gen_struct(self, s: &Struct) -> Vec<(ModName, Block)> {
        vec![(self.mod_name(s), self.gen_struct_owned(s))]
    }
}
