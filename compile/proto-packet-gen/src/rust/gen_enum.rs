use crate::rust::derives::add_derives;
use crate::rust::docs::add_docs;
use crate::rust::naming::{ident, is_upper_camel_case};
use crate::rust::prelude::{Prelude, PreludeName};
use code_gen::rust::{
    Access, Enum as RustEnum, EnumCase, EnumFields, Function, ImplBlock, MatchCase, MatchStatement,
    Receiver, Signature, Var, WithAccess, WithAttributes, WithComments, WithFnGenerics,
    WithFunctions, WithGenerics, WithReceiver, WithResult, WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{Enum, WithCaseName, WithComments as _, WithTypeName};

/// Generates the Rust code of the `enumeration`.
///
/// Each enum has an `Unrecognized` case for the tag numbers it doesn't know. Its tag number is in a
/// private module, so only `from_tag_number` can create it.
///
/// The serde impls are written by hand, since they go through `from_tag_number`. A case is its
/// schema name, even when renamed, & an unrecognized case is its tag number. (ex: `Self_` is `Self`)
pub fn gen_enum(enumeration: &Enum) -> String {
    let prelude: Prelude = Prelude::new(&ident(&enumeration.type_name()));
    let dec: Block = Block::default().with_statement(gen_dec(enumeration));
    let block: Block = Block::default()
        .with_statement(gen_construction(prelude, enumeration))
        .with_statement(EmptyLine::default())
        .with_statement(gen_ord(prelude, enumeration))
        .with_statement(EmptyLine::default())
        .with_statement(gen_partial_ord(prelude, enumeration))
        .with_statement(EmptyLine::default())
        .with_statement(gen_properties(enumeration))
        .with_statement(EmptyLine::default())
        .with_statement(gen_serialize(prelude, enumeration))
        .with_statement(EmptyLine::default())
        .with_statement(gen_deserialize(prelude, enumeration));
    format!(
        "{}\n{}\n{}",
        dec.to_code(),
        UNRECOGNIZED_MOD,
        block.to_code()
    )
}

/// The private module with the tag number of an unrecognized case.
const UNRECOGNIZED_MOD: &str = "\
mod unrecognized {
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct UnrecognizedTagNumber(pub(super) u16);
}
";

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
    let unrecognized: EnumFields =
        EnumFields::Unnamed(vec!["unrecognized::UnrecognizedTagNumber".into()]);
    result.add_case(
        EnumCase::from("Unrecognized")
            .with_comment(" A tag number the enum doesn't know.")
            .with_fields(unrecognized),
    );
    for case in enumeration.cases() {
        let mut rust_case: EnumCase = EnumCase::from(ident(&case.case_name()));
        add_docs(&mut rust_case, case.comments());
        result.add_case(rust_case);
    }
    result
}

/// Generates the impl block creating the cases of the `enumeration` from tag numbers.
///
/// `Some` & `None` are bare, since generated types are never values, so they cannot shadow them.
fn gen_construction(prelude: Prelude, enumeration: &Enum) -> ImplBlock {
    let mut cases: MatchStatement = MatchStatement::from("tag_number.value()");
    for case in enumeration.cases() {
        cases.add_match_case(
            MatchCase::from(case.tag_number().to_string())
                .with_literal(format!("Self::{}", ident(&case.case_name()))),
        );
    }
    let signature: Signature = Signature::from("from_tag_number")
        .with_param(Var::from(("tag_number", "proto_packet::io::TagNumber")))
        .with_result("Self");
    let mut from: Function = Function::from(signature)
        .with_comment(" Gets the case with the `tag_number`.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true);
    if enumeration.cases().is_empty() {
        from.add_literal(
            "Self::Unrecognized(unrecognized::UnrecognizedTagNumber(tag_number.value()))",
        );
    } else {
        from.add_statement(cases.with_match_case(
            MatchCase::from("tag_number").with_literal(
                "Self::Unrecognized(unrecognized::UnrecognizedTagNumber(tag_number))",
            ),
        ));
    }

    let signature: Signature = Signature::from("from_tag_number_optional")
        .with_param(Var::from(("tag_number", "u32")))
        .with_result(format!("{}<Self>", prelude.path(PreludeName::Option)));
    let optional: MatchStatement =
        MatchStatement::from("proto_packet::io::TagNumber::from_u32(tag_number)")
            .with_match_case(
                MatchCase::from("Some(tag_number)")
                    .with_literal("Some(Self::from_tag_number(tag_number))"),
            )
            .with_match_case(MatchCase::from("None").with_literal("None"));
    let from_optional: Function = Function::from(signature)
        .with_comment(" Gets the case with the `tag_number`.")
        .with_comment("")
        .with_comment(" Returns `None` if the `tag_number` is invalid.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true)
        .with_statement(optional);

    ImplBlock::from(ident(&enumeration.type_name()))
        .with_comment(" Construction")
        .with_function(from)
        .with_function(from_optional)
}

/// Generates the `Ord` impl of the `enumeration`, ordering its cases by tag number instead of
/// declaration order.
fn gen_ord(prelude: Prelude, enumeration: &Enum) -> ImplBlock {
    let signature: Signature = Signature::from("cmp")
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from(("other", "&Self")))
        .with_result("std::cmp::Ordering");
    let cmp: Function =
        Function::from(signature).with_literal("self.tag_number().cmp(&other.tag_number())");
    ImplBlock::from(ident(&enumeration.type_name()))
        .with_for_trait(prelude.path(PreludeName::Ord))
        .with_function(cmp)
}

/// Generates the `PartialOrd` impl of the `enumeration`, consistent with its `Ord` impl.
fn gen_partial_ord(prelude: Prelude, enumeration: &Enum) -> ImplBlock {
    let signature: Signature = Signature::from("partial_cmp")
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from(("other", "&Self")))
        .with_result(format!(
            "{}<std::cmp::Ordering>",
            prelude.path(PreludeName::Option)
        ));
    let partial_cmp: Function = Function::from(signature).with_literal("Some(self.cmp(other))");
    ImplBlock::from(ident(&enumeration.type_name()))
        .with_for_trait(prelude.path(PreludeName::PartialOrd))
        .with_function(partial_cmp)
}

/// Generates the impl block with the tag numbers of the cases of the `enumeration`.
fn gen_properties(enumeration: &Enum) -> ImplBlock {
    let mut to_tag_number: MatchStatement = MatchStatement::from("self").with_match_case(
        MatchCase::from("Self::Unrecognized(unrecognized)").with_literal("unrecognized.0"),
    );
    for case in enumeration.cases() {
        to_tag_number.add_match_case(
            MatchCase::from(format!("Self::{}", ident(&case.case_name())))
                .with_literal(case.tag_number().to_string()),
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
    ImplBlock::from(ident(&enumeration.type_name()))
        .with_comment(" Properties")
        .with_function(to)
}

/// Generates the `Serialize` impl of the `enumeration`, writing each case as its schema name, & an
/// unrecognized case as its tag number.
fn gen_serialize(prelude: Prelude, enumeration: &Enum) -> ImplBlock {
    let signature: Signature = Signature::from("serialize")
        .with_generic(Var::from(("S", "proto_packet::serde::Serializer")))
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from(("serializer", "S")))
        .with_result(format!(
            "{}<S::Ok, S::Error>",
            prelude.path(PreludeName::Result)
        ));
    let mut cases: MatchStatement = MatchStatement::from("self").with_match_case(
        MatchCase::from("Self::Unrecognized(unrecognized)")
            .with_literal("serializer.serialize_u16(unrecognized.0)"),
    );
    for case in enumeration.cases() {
        cases.add_match_case(
            MatchCase::from(format!("Self::{}", ident(&case.case_name()))).with_literal(format!(
                "serializer.serialize_str(\"{}\")",
                case.case_name()
            )),
        );
    }
    ImplBlock::from(ident(&enumeration.type_name()))
        .with_for_trait("proto_packet::serde::Serialize")
        .with_function(Function::from(signature).with_statement(cases))
}

/// Generates the `Deserialize` impl of the `enumeration`, reading each case from its schema name or
/// its tag number. (with `from_tag_number`)
fn gen_deserialize(prelude: Prelude, enumeration: &Enum) -> ImplBlock {
    let signature: Signature = Signature::from("deserialize")
        .with_generic(Var::from(("D", "proto_packet::serde::Deserializer<'de>")))
        .with_param(Var::from(("deserializer", "D")))
        .with_result(format!(
            "{}<Self, D::Error>",
            prelude.path(PreludeName::Result)
        ));
    let from_name: String = if enumeration.cases().is_empty() {
        "|_| None".to_string()
    } else {
        let cases: Vec<String> = enumeration
            .cases()
            .iter()
            .map(|case| {
                format!(
                    "\"{}\" => Some(Self::{})",
                    case.case_name(),
                    ident(&case.case_name())
                )
            })
            .collect();
        format!("|name| match name {{ {}, _ => None }}", cases.join(", "))
    };
    let deserialize: Function = Function::from(signature).with_literal(format!(
        "proto_packet::types::deserialize_enum(deserializer, \"{}\", {}, Self::from_tag_number_optional)",
        enumeration.type_name(),
        from_name
    ));
    ImplBlock::from(ident(&enumeration.type_name()))
        .with_generic("'de")
        .with_for_trait("proto_packet::serde::Deserialize<'de>")
        .with_function(deserialize)
}
