use crate::rust::derives::add_derives;
use crate::rust::docs::add_docs;
use crate::rust::naming::{ident, is_upper_camel_case};
use crate::rust::typing::Typing;
use code_gen::Statement;
use code_gen::rust::{Access, Enum as RustEnum, EnumCase, EnumFields, WithAccess, WithAttributes};
use proto_packet_tree::{Project, Variant, WithCaseName, WithComments, WithTypeName, WithTypeTag};

/// Generates the Rust code of the `variant`.
pub fn gen_variant(project: &Project, variant: &Variant) -> String {
    let typing: Typing = Typing::new(project);
    let mut result: RustEnum = RustEnum::from(ident(&variant.type_name()));
    add_docs(&mut result, variant.comments());
    add_derives(&mut result, false, false);
    result.add_attribute("serde(tag = \"var\", content = \"val\")");
    let is_camel_case: bool = is_upper_camel_case(&variant.type_name())
        && variant
            .cases()
            .iter()
            .all(|case| is_upper_camel_case(&case.case_name()));
    if !is_camel_case {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    for case in variant.cases() {
        let fields: EnumFields = EnumFields::Unnamed(vec![typing.storage_type(case.type_tag())]);
        let mut rust_case: EnumCase = EnumCase::from(ident(&case.case_name())).with_fields(fields);
        add_docs(&mut rust_case, case.comments());
        result.add_case(rust_case);
    }
    result.to_code()
}
