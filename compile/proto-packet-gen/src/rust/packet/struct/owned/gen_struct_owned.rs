use crate::rust::GenRust;
use code_gen::{Block, WithStatements};
use proto_packet_tree::Struct;

impl GenRust<'_> {
    //! Gen Struct: Owned

    pub(in super::super) fn gen_struct_owned(self, s: &Struct) -> Block {
        Block::default()
            .with_statement(self.gen_struct_owned_dec(s))
            .with_empty_line()
            .with_statement(self.gen_struct_owned_construct(s))
    }
}
