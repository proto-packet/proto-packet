use crate::rust::GenRust;
use code_gen::rust::Access::Public;
use code_gen::rust::{RustType, Struct as RustStruct, WithAccess, WithStructFields};
use proto_packet_tree::Struct;

impl GenRust<'_> {
    //! Gen Struct: Owned Declaration

    pub(super) fn gen_struct_owned_dec(self, s: &Struct) -> RustStruct {
        let mut result: RustStruct = RustStruct::from(self.type_name(s));
        self.gen_comments_type_dec("struct", s, &mut result, |s, result| {
            self.gen_comments_fields(
                s.fields(),
                &s.fields().iter().map(|_| None).collect::<Vec<_>>(),
                result,
            )
        });
        self.gen_derives(self.is_copy_struct(s), false, &mut result);
        result.set_access(Public);
        for field in s.fields() {
            let name: String = self.field_name(field);
            let tag: RustType = self.field_type(field, false);
            result.add_field((name, tag));
        }
        result
    }
}
