use crate::rust::GenRust;
use code_gen::Block;
use proto_packet_tree::{ModName, Variant};

impl GenRust<'_> {
    //! Gen Variant

    pub(in crate::rust) fn gen_variant(self, v: &Variant) -> Vec<(ModName, Block)> {
        vec![(self.mod_name(v), self.gen_variant_owned(v))]
    }
}
