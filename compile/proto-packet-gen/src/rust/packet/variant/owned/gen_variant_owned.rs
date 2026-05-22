use crate::rust::GenRust;
use code_gen::{Block, WithStatements};
use proto_packet_tree::Variant;

impl GenRust<'_> {
    //! Gen Variant: Owned

    pub(in super::super) fn gen_variant_owned(self, v: &Variant) -> Block {
        Block::default().with_statement(self.gen_variant_owned_dec(v))
    }
}
