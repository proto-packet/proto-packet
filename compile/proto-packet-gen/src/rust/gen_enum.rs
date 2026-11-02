use crate::rust::derives::add_derives;
use crate::rust::docs::add_docs;
use crate::rust::naming::{ident, is_renamed, is_upper_camel_case};
use code_gen::rust::{
    Access, Enum as RustEnum, EnumCase, Function, ImplBlock, MatchCase, MatchStatement, Receiver,
    Signature, Var, WithAccess, WithAttributes, WithComments, WithFunctions, WithReceiver,
    WithResult, WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{Enum, WithCaseName, WithComments as _, WithTypeName};

/// Generates the Rust code of the `enumeration`.
///
/// A renamed case keeps its schema name in serde. (ex: `Self_` is `Self`)
pub fn gen_enum(enumeration: &Enum) -> String {
    let block: Block = Block::default()
        .with_statement(gen_dec(enumeration))
        .with_statement(EmptyLine::default())
        .with_statement(gen_tag_numbers(enumeration))
        .with_statement(EmptyLine::default())
        .with_statement(gen_ord(enumeration))
        .with_statement(EmptyLine::default())
        .with_statement(gen_partial_ord(enumeration));
    block.to_code()
}

/// Generates the declaration of the `enumeration`.
fn gen_dec(enumeration: &Enum) -> RustEnum {
    let mut result: RustEnum = RustEnum::from(ident(&enumeration.type_name()));
    add_docs(&mut result, enumeration.comments());
    add_derives(&mut result, true, false, false);
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
        if is_renamed(&case.case_name()) {
            rust_case.add_attribute(format!("serde(rename = \"{}\")", case.case_name()));
        }
        result.add_case(rust_case);
    }
    result
}

/// Generates the impl block converting the cases of the `enumeration` to & from tag numbers.
fn gen_tag_numbers(enumeration: &Enum) -> ImplBlock {
    let mut to_tag_number: MatchStatement = MatchStatement::from("self");
    let mut from_tag_number: MatchStatement = MatchStatement::from("tag_number");
    for case in enumeration.cases() {
        let path: String = format!("Self::{}", ident(&case.case_name()));
        let tag_number: String = case.tag_number().to_string();
        to_tag_number
            .add_match_case(MatchCase::from(path.as_str()).with_literal(tag_number.as_str()));
        from_tag_number.add_match_case(
            MatchCase::from(tag_number)
                .with_literal(format!("::std::option::Option::Some({})", path)),
        );
    }

    let signature: Signature = Signature::from("tag_number")
        .with_receiver(Receiver::Owned)
        .with_result("u16");
    let to: Function = Function::from(signature)
        .with_comment(" Gets the tag number.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true)
        .with_statement(to_tag_number);

    let param: &str = if enumeration.cases().is_empty() {
        "_tag_number"
    } else {
        "tag_number"
    };
    let signature: Signature = Signature::from("from_tag_number")
        .with_param(Var::from((param, "u16")))
        .with_result("::std::option::Option<Self>");
    let mut from: Function = Function::from(signature)
        .with_comment(" Gets the optional case with the `tag_number`.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true);
    if enumeration.cases().is_empty() {
        from.add_literal("::std::option::Option::None");
    } else {
        from.add_statement(
            from_tag_number
                .with_match_case(MatchCase::from("_").with_literal("::std::option::Option::None")),
        );
    }

    ImplBlock::from(ident(&enumeration.type_name()))
        .with_comment(" Tag Numbers")
        .with_function(to)
        .with_function(from)
}

/// Generates the `Ord` impl of the `enumeration`, ordering its cases by tag number instead of
/// declaration order. (see `FORMAT.md`)
fn gen_ord(enumeration: &Enum) -> ImplBlock {
    let signature: Signature = Signature::from("cmp")
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from(("other", "&Self")))
        .with_result("::std::cmp::Ordering");
    let cmp: Function =
        Function::from(signature).with_literal("self.tag_number().cmp(&other.tag_number())");
    ImplBlock::from(ident(&enumeration.type_name()))
        .with_for_trait("::std::cmp::Ord")
        .with_function(cmp)
}

/// Generates the `PartialOrd` impl of the `enumeration`, consistent with its `Ord` impl.
fn gen_partial_ord(enumeration: &Enum) -> ImplBlock {
    let signature: Signature = Signature::from("partial_cmp")
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from(("other", "&Self")))
        .with_result("::std::option::Option<::std::cmp::Ordering>");
    let partial_cmp: Function =
        Function::from(signature).with_literal("::std::option::Option::Some(self.cmp(other))");
    ImplBlock::from(ident(&enumeration.type_name()))
        .with_for_trait("::std::cmp::PartialOrd")
        .with_function(partial_cmp)
}
