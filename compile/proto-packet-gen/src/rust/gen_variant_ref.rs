use crate::rust::encoding::IO;
use crate::rust::gen_buf::buf_name;
use crate::rust::gen_forms::{gen_new, gen_new_unchecked, gen_to_buf};
use crate::rust::gen_message_ref::{gen_read, gen_value_check};
use crate::rust::gen_struct_ref::{gen_ref_dec, gen_ref_properties, ref_name};
use crate::rust::gen_variant::Case;
use crate::rust::naming::is_upper_camel_case;
use crate::rust::typing::Typing;
use crate::rust::values::ref_value_type;
use code_gen::rust::{
    Access, Enum as RustEnum, EnumCase, EnumFields, Function, ImplBlock, Receiver, RustType,
    Signature, WithAccess, WithAttributes, WithComments, WithDerives, WithFunctions, WithGenerics,
    WithReceiver, WithResult,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};

/// Gets the name of the `Case` type of the variant with the schema `type_name`.
///
/// (ex: `FooCase` for `Foo`)
pub(crate) fn case_name(type_name: &str) -> String {
    format!("{}Case", type_name)
}

/// Generates the `Case` type of the variant with the `schema_name` & the Rust `type_name`.
///
/// It's the value of a `Ref` type as an enum, so it can be matched without copying.
pub(crate) fn gen_variant_case(
    typing: Typing<'_>,
    schema_name: &str,
    cases: &[Case<'_>],
    type_name: &str,
) -> String {
    let mut result: RustEnum = RustEnum::from(case_name(schema_name))
        .with_comment(format!(
            " A case of an encoded [{}](super::{}).",
            type_name, type_name
        ))
        .with_generic("'a");
    for derive in ["Copy", "Clone", "Debug"] {
        result.add_derive(derive);
    }
    let is_camel_case: bool = is_upper_camel_case(schema_name)
        && cases.iter().all(|case| is_upper_camel_case(&case.name));
    if !is_camel_case {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    result.add_case(
        EnumCase::from("Unrecognized")
            .with_comment(" A case the variant doesn't know.")
            .with_fields(EnumFields::Named(vec![
                ("tag_number", "u16").into(),
                ("bytes", "&'a [u8]").into(),
            ])),
    );
    for case in cases {
        let fields: EnumFields = EnumFields::Unnamed(vec![ref_value_type(typing, case.tag).into()]);
        result.add_case(
            EnumCase::from(case.ident())
                .with_comment(format!(" The case: `{}`.", case.name))
                .with_fields(fields),
        );
    }
    result.to_code()
}

/// Generates the `Ref` type of the variant with the `schema_name` & the Rust `type_name`.
///
/// It borrows the valid encoding of the variant. Its file imports nothing & declares only the `Ref`
/// type, so the prelude names are never shadowed.
pub(crate) fn gen_variant_ref(
    typing: Typing<'_>,
    schema_name: &str,
    cases: &[Case<'_>],
    type_name: &str,
) -> String {
    let name: String = ref_name(schema_name);
    let ref_type: RustType = RustType::from(name.as_str()).with_generic(RustType::from("'a"));
    let mut block: Block = Block::default()
        .with_statement(gen_ref_dec(schema_name, &name, type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_construction(typing, cases, &ref_type))
        .with_statement(EmptyLine::default())
        .with_statement(gen_ref_properties(&ref_type))
        .with_statement(EmptyLine::default())
        .with_statement(gen_cases(typing, schema_name, cases, &ref_type));
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_decoding(
        typing,
        schema_name,
        cases,
        &ref_type,
        type_name,
    ));
    block.to_code()
}

/// Generates the impl block constructing the `ref_type`.
fn gen_construction(typing: Typing<'_>, cases: &[Case<'_>], ref_type: &RustType) -> ImplBlock {
    let mut new: Function = gen_new("ref", "&'a [u8]")
        .with_semi(format!("{}::check_len(bytes, 2)?", IO))
        .with_semi(format!(
            "let tag_number: u16 = unsafe {{ {}::read_u16_unchecked(bytes, 0) }}",
            IO
        ))
        .with_semi(format!("{}::check_tag_number(tag_number)?", IO));
    if !cases.is_empty() {
        new.add_semi("let value: &'a [u8] = unsafe { bytes.get_unchecked(2..) }");
        new.add_literal(if let [case] = cases {
            format!(
                "if tag_number == {} {{ {} }}",
                case.number,
                gen_value_check(typing, case.tag, case.size)
            )
        } else {
            let arms: Vec<String> = cases
                .iter()
                .map(|case| {
                    format!(
                        "{} => {{ {} }}",
                        case.number,
                        gen_value_check(typing, case.tag, case.size)
                    )
                })
                .collect();
            format!("match tag_number {{ {} _ => {{}} }}", arms.join(" "))
        });
    }
    let new: Function = new.with_literal("Ok(Self { bytes })");

    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Construction")
        .with_function(new)
        .with_function(gen_new_unchecked("ref", "&'a [u8]"))
}

/// Generates the impl block with the tag number, the case & the getter of each of the `cases`.
fn gen_cases(
    typing: Typing<'_>,
    schema_name: &str,
    cases: &[Case<'_>],
    ref_type: &RustType,
) -> ImplBlock {
    let signature: Signature = Signature::from("tag_number")
        .with_receiver(Receiver::Borrowed)
        .with_result("u16");
    let tag_number: Function = Function::from(signature)
        .with_comment(" Gets the tag number of the case.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ {}::read_u16_unchecked(self.bytes, 0) }}",
            IO
        ));

    let case_type: String = format!("super::{}", case_name(schema_name));
    let unrecognized: String =
        format!("{}::Unrecognized {{ tag_number, bytes: value }}", case_type);
    let mut arms: Vec<String> = cases
        .iter()
        .map(|case| {
            format!(
                "{} => {}::{}({}),",
                case.number,
                case_type,
                case.ident(),
                gen_read(typing, case.tag)
            )
        })
        .collect();
    arms.push(format!("tag_number => {},", unrecognized));
    let signature: Signature = Signature::from("case")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("{}<'a>", case_type));
    let mut case: Function = Function::from(signature)
        .with_comment(" Gets the case, to match on it.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_semi("let value: &'a [u8] = unsafe { self.bytes.get_unchecked(2..) }");
    if cases.is_empty() {
        case.add_semi("let tag_number: u16 = self.tag_number()");
        case.add_literal(unrecognized);
    } else {
        case.add_literal(format!("match self.tag_number() {{ {} }}", arms.join(" ")));
    }

    let mut result: ImplBlock = ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Cases")
        .with_function(tag_number)
        .with_function(case);
    for case in cases {
        let signature: Signature = Signature::from(case.getter_name())
            .with_receiver(Receiver::Borrowed)
            .with_result(format!("Option<{}>", ref_value_type(typing, case.tag)));
        result.add_function(
            Function::from(signature)
                .with_comment(format!(" Gets the value of the case: `{}`.", case.name))
                .with_attribute("must_use")
                .with_access(Access::Public)
                .with_literal(format!(
                    "if self.tag_number() != {} {{ return None; }}",
                    case.number
                ))
                .with_semi("let value: &'a [u8] = unsafe { self.bytes.get_unchecked(2..) }")
                .with_literal(format!("Some({})", gen_read(typing, case.tag))),
        );
    }
    result
}

/// Generates the impl block decoding the `ref_type` of the variant with the `type_name`.
fn gen_decoding(
    typing: Typing<'_>,
    schema_name: &str,
    cases: &[Case<'_>],
    ref_type: &RustType,
    type_name: &str,
) -> ImplBlock {
    let case_type: String = format!("super::{}", case_name(schema_name));
    let mut arms: Vec<String> = vec![format!(
        "{}::Unrecognized {{ tag_number, bytes }} => super::{}::from_unknown(tag_number, bytes.to_vec()),",
        case_type, type_name
    )];
    for case in cases {
        arms.push(format!(
            "{}::{}(value) => super::{}::{}({}),",
            case_type,
            case.ident(),
            type_name,
            case.ident(),
            gen_decoded_value(typing, *case)
        ));
    }
    let signature: Signature = Signature::from("decode")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("super::{}", type_name));
    let decode: Function = Function::from(signature)
        .with_comment(format!(
            " Decodes the [{}](super::{}).",
            type_name, type_name
        ))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!("match self.case() {{ {} }}", arms.join(" ")));

    let buf: String = buf_name(schema_name);

    ImplBlock::from(ref_type.clone())
        .with_generic("'a")
        .with_comment(" Decoding")
        .with_function(decode)
        .with_function(gen_to_buf(&buf))
}

/// Generates the expression of the owned `value` of the `case`.
fn gen_decoded_value(typing: Typing<'_>, case: Case<'_>) -> String {
    let value: &str = if case.tag.is_string() {
        "value.to_string()"
    } else if case.is_nested(typing) || case.tag.is_list() {
        "value.decode()"
    } else {
        "value"
    };
    if case.is_boxed {
        format!("Box::new({})", value)
    } else {
        value.to_string()
    }
}
