use crate::rust::GenRust;
use code_gen::rust::Access::Public;
use code_gen::rust::{
    Enum as RustEnum, EnumCase as RustEnumCase, WithAccess, WithComments as RustWithComments,
};
use proto_packet_tree::{Enum, WithComments};

impl GenRust<'_> {
    //! Gen Enum: Owned Declaration

    pub(super) fn gen_enum_owned_dec(self, e: &Enum) -> RustEnum {
        let mut result: RustEnum = RustEnum::from(self.type_name(e));
        for comment in e.comments() {
            result.add_comment(comment);
        }
        self.gen_derives(true, false, &mut result);
        result.set_access(Public);
        for case in e.cases() {
            let mut rust_case: RustEnumCase = RustEnumCase::from(self.case_name(case));
            for comment in case.comments() {
                rust_case.add_comment(comment);
            }
            result.add_case(rust_case);
        }
        result
    }
}
