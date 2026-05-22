use crate::rust::GenRust;
use code_gen::rust::Access::Public;
use code_gen::rust::{
    Enum as RustEnum, EnumCase as RustEnumCase, EnumFields, RustType, WithAccess,
    WithComments as RustWithComments,
};
use proto_packet_tree::{Variant, WithComments};

impl GenRust<'_> {
    //! Gen Variant: Owned Declaration

    pub(super) fn gen_variant_owned_dec(self, v: &Variant) -> RustEnum {
        let mut result: RustEnum = RustEnum::from(self.type_name(v));
        for comment in v.comments() {
            result.add_comment(comment);
        }
        self.gen_derives(false, false, &mut result);
        result.set_access(Public);
        for case in v.cases() {
            let tag: RustType = self.field_type(case, false);
            let mut rust_case: RustEnumCase = RustEnumCase::from(self.case_name(case))
                .with_fields(EnumFields::Unnamed(vec![tag]));
            for comment in case.comments() {
                rust_case.add_comment(comment);
            }
            result.add_case(rust_case);
        }
        result
    }
}
