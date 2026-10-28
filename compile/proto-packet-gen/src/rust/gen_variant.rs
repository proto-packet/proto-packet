use crate::rust::derives::add_derives;
use crate::rust::docs::add_docs;
use crate::rust::naming::{ident, is_renamed, is_upper_camel_case};
use crate::rust::typing::Typing;
use code_gen::Statement;
use code_gen::rust::{
    Access, Enum as RustEnum, EnumCase, EnumFields, RustType, WithAccess, WithAttributes,
};
use proto_packet_tree::{Project, Variant, WithCaseName, WithComments, WithTypeName, WithTypeTag};

/// Generates the Rust code of the `variant`.
///
/// A renamed case keeps its schema name in serde. (ex: `Self_` is `Self`)
pub fn gen_variant(project: &Project, variant: &Variant) -> String {
    let typing: Typing = Typing::new(project);
    let mut result: RustEnum = RustEnum::from(ident(&variant.type_name()));
    add_docs(&mut result, variant.comments());
    add_derives(&mut result, false, true, false);
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
        let mut case_type: RustType = typing.storage_type(case.type_tag());
        if case.is_boxed() {
            case_type = RustType::from("::std::boxed::Box").with_generic(case_type);
        }
        let fields: EnumFields = EnumFields::Unnamed(vec![case_type]);
        let mut rust_case: EnumCase = EnumCase::from(ident(&case.case_name())).with_fields(fields);
        add_docs(&mut rust_case, case.comments());
        if is_renamed(&case.case_name()) {
            rust_case.add_attribute(format!("serde(rename = \"{}\")", case.case_name()));
        }
        result.add_case(rust_case);
    }
    result.to_code()
}
