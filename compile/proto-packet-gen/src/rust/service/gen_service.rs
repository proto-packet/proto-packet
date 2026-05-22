use crate::rust::GenRust;
use crate::rust::service::raw_statement::RawStatement;
use code_gen::{Block, WithStatements};
use proto_packet_tree::{ModName, ModPathRef, Service};

impl GenRust<'_> {
    //! Gen Service

    pub(in crate::rust) fn gen_service(
        self,
        schema_path: ModPathRef<'_>,
        s: &Service,
    ) -> Vec<(ModName, Block)> {
        let block: Block = Block::default()
            .with_statement(RawStatement(self.gen_service_trait_text(s)))
            .with_empty_line()
            .with_statement(RawStatement(self.gen_service_router_text(schema_path, s)));
        vec![(self.mod_name(s), block)]
    }
}
