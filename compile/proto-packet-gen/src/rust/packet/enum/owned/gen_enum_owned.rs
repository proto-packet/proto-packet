use crate::rust::GenRust;
use code_gen::{Block, WithStatements};
use proto_packet_tree::Enum;

impl GenRust<'_> {
    //! Gen Enum: Owned

    pub(in super::super) fn gen_enum_owned(self, e: &Enum) -> Block {
        Block::default().with_statement(self.gen_enum_owned_dec(e))
    }
}
