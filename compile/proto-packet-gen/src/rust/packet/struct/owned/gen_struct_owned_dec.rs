use crate::rust::GenRust;
use code_gen::rust::Access::Public;
use code_gen::rust::{RustType, Struct as RustStruct, WithAccess, WithStructFields};
use proto_packet_tree::Struct;

impl GenRust<'_> {
    //! Gen Struct: Owned Declaration

    pub(super) fn gen_struct_owned_dec(self, s: &Struct) -> RustStruct {
        let mut result: RustStruct = RustStruct::from(self.type_name(s));
        result.set_access(Public);
        for field in s.fields() {
            let name: String = self.field_name(field);
            let tag: RustType = self.field_type(field, false);
            result.add_field((name, tag));
        }
        result
    }
}
