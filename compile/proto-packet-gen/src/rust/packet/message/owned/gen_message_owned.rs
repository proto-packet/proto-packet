use crate::rust::GenRust;
use code_gen::{Block, WithStatements};
use proto_packet_tree::Message;

impl GenRust<'_> {
    //! Gen Message: Owned

    pub(in super::super) fn gen_message_owned(self, m: &Message) -> Block {
        let mut block: Block = Block::default().with_statement(self.gen_message_owned_dec(m));
        for field in m.fields() {
            block.add_empty_line();
            block.add_statement(self.gen_field_impl(m, field, true))
        }
        block
    }
}
