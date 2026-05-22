use crate::rust::GenRust;
use code_gen::Block;
use proto_packet_tree::{Message, ModName};

impl GenRust<'_> {
    //! Gen Message

    pub(in crate::rust) fn gen_message(self, m: &Message) -> Vec<(ModName, Block)> {
        vec![(self.mod_name(m), self.gen_message_owned(m))]
    }
}
