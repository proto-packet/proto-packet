use crate::rust::GenRust;
use code_gen::rust::Access::Public;
use code_gen::rust::{RustType, Struct as RustStruct, WithAccess, WithStructFields};
use proto_packet::io::WithTagNumber;
use proto_packet_tree::Message;

impl GenRust<'_> {
    //! Gen Message: Owned Declaration

    pub(super) fn gen_message_owned_dec(self, m: &Message) -> RustStruct {
        let mut result: RustStruct = RustStruct::from(self.type_name(m));
        self.gen_comments_type_dec("message", m, &mut result, |m, result| {
            self.gen_comments_fields(
                m.fields(),
                &m.fields()
                    .iter()
                    .map(|f| Some(f.tag_number()))
                    .collect::<Vec<_>>(),
                result,
            )
        });
        self.gen_derives(false, true, &mut result);
        result.set_access(Public);
        for field in m.fields() {
            let name: String = self.field_name(field);
            let tag: RustType = self.field_type(field, true);
            result.add_field((name, tag));
        }
        result
    }
}
