use crate::rust::derives::add_derives;
use crate::rust::docs::add_docs;
use crate::rust::naming::{ident, is_upper_camel_case};
use code_gen::Statement;
use code_gen::rust::{Access, Enum as RustEnum, EnumCase, WithAccess, WithAttributes};
use proto_packet_tree::{Enum, WithCaseName, WithComments, WithTypeName};

/// Generates the Rust code of the `enumeration`.
pub fn gen_enum(enumeration: &Enum) -> String {
    let mut result: RustEnum = RustEnum::from(ident(&enumeration.type_name()));
    add_docs(&mut result, enumeration.comments());
    add_derives(&mut result, true, false);
    let is_camel_case: bool = is_upper_camel_case(&enumeration.type_name())
        && enumeration
            .cases()
            .iter()
            .all(|case| is_upper_camel_case(&case.case_name()));
    if !is_camel_case {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    for case in enumeration.cases() {
        let mut rust_case: EnumCase = EnumCase::from(ident(&case.case_name()));
        add_docs(&mut rust_case, case.comments());
        result.add_case(rust_case);
    }
    result.to_code()
}
