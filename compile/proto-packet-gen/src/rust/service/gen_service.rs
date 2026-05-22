use crate::rust::GenRust;
use code_gen::{Block, WithStatements};
use proto_packet_tree::{ModName, Service};

impl GenRust<'_> {
    //! Gen Service

    pub(in crate::rust) fn gen_service(self, s: &Service) -> Vec<(ModName, Block)> {
        let block: Block = Block::default().with_statement(self.gen_service_dec(s));
        vec![(self.mod_name(s), block)]
    }
}
