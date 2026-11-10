use crate::GenError;
use crate::layout::scalar_size;
use crate::rust::TypeFile;
use crate::rust::derives::add_derives;
use crate::rust::docs::add_docs;
use crate::rust::encoding::{IO, gen_encode, gen_encode_to_of_level, gen_encode_to_with_depth};
use crate::rust::gen_buf::{buf_name, gen_buf};
use crate::rust::gen_element::gen_element;
use crate::rust::gen_eq::gen_native_eq;
use crate::rust::gen_json::{SERDE, TYPES, gen_serde_impls};
use crate::rust::gen_ord::gen_variant_ord;
use crate::rust::gen_struct::{check_list_elements, check_type_name, gen_decoding};
use crate::rust::gen_struct_mut::mut_name;
use crate::rust::gen_struct_ref::ref_name;
use crate::rust::gen_variant_mut::{gen_variant_buf_accessors, gen_variant_mut};
use crate::rust::gen_variant_ref::{case_name, gen_variant_case, gen_variant_ref};
use crate::rust::gen_variant_view::{gen_variant_view, gen_variant_view_mut};
use crate::rust::gen_view::view_name;
use crate::rust::gen_view_mut::view_mut_name;
use crate::rust::message_encoding::collection;
use crate::rust::message_encoding::gen_value_write;
use crate::rust::naming::{ident, is_upper_camel_case, snake_case};
use crate::rust::prelude::{Prelude, PreludeName};
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Enum as RustEnum, EnumCase, EnumFields, Function, ImplBlock, Receiver, RustType,
    Signature, Var, WithAccess, WithAttributes, WithComments, WithFunctions, WithReceiver,
    WithResult, WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{
    CaseNameRef, ModPathRef, Project, TypeTag, Variant, VariantCase, WithCaseName,
    WithComments as _, WithTypeName, WithTypeTag,
};
use std::collections::BTreeSet;

/// The private module with the tag number & bytes of an unrecognized case.
const UNRECOGNIZED_MOD: &str = "\
mod unrecognized {
    #[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
    pub struct UnrecognizedCase {
        pub(super) tag_number: u16,
        pub(super) bytes: Vec<u8>,
    }
}
";

/// A case of a generated variant.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Case<'a> {
    pub(crate) name: CaseNameRef<'a>,
    pub(crate) tag: &'a TypeTag,
    pub(crate) number: u16,
    pub(crate) size: Option<usize>,
    pub(crate) is_boxed: bool,
}

impl Case<'_> {
    //! Names

    /// Gets the name of the Rust case. (ex: `Circle`)
    pub(crate) fn ident(self) -> String {
        ident(&self.name)
    }

    /// Gets the name of the case's getter. (ex: `as_circle`)
    pub(crate) fn getter_name(self) -> String {
        format!("as_{}", snake_case(&self.name))
    }

    /// Gets the name of the case's mutable accessor. (ex: `as_square_mut`)
    pub(crate) fn accessor_name(self) -> String {
        format!("as_{}_mut", snake_case(&self.name))
    }

    /// Gets the name of the case's setter. (ex: `set_circle`)
    pub(crate) fn setter_name(self) -> String {
        format!("set_{}", snake_case(&self.name))
    }
}

impl Case<'_> {
    //! Kinds

    /// Checks if the case's value is a nested struct, message or variant.
    pub(crate) fn is_nested(self, typing: Typing<'_>) -> bool {
        matches!(self.tag, TypeTag::Named(_)) && !typing.is_enum(self.tag)
    }
}

/// Gets the case of the `case`.
fn case<'a>(project: &Project, case: &'a VariantCase) -> Case<'a> {
    Case {
        name: case.case_name(),
        tag: case.type_tag(),
        number: case.tag_number().value(),
        size: scalar_size(project, case.type_tag()),
        is_boxed: case.is_boxed(),
    }
}

/// Generates the Rust files of the `variant` in the schema file at the `mod_path`.
///
/// Each variant has an `Unrecognized` case for the tag numbers it doesn't know, with their bytes.
/// The bytes are in a private module, so only the variant's own file can create that case. A
/// renamed case keeps its schema name in serde. (ex: `Self_` is `Self`)
pub fn gen_variant(
    project: &Project,
    mod_path: ModPathRef<'_>,
    variant: &Variant,
) -> Result<Vec<TypeFile>, GenError> {
    let type_name: String = ident(&variant.type_name());
    let typing: Typing = Typing::new(project, Prelude::new(&type_name));
    let cases: Vec<Case> = variant
        .cases()
        .iter()
        .map(|variant_case| case(project, variant_case))
        .collect();
    check_case_names(
        typing,
        &format!("{}.{}", mod_path, variant.type_name()),
        &cases,
    )?;
    check_list_elements(
        project,
        &format!("{}.{}", mod_path, variant.type_name()),
        cases.iter().map(|case| (case.name.value(), case.tag)),
    )?;

    let inception: &str = if snake_case(&variant.type_name()) == "unrecognized" {
        "#[allow(clippy::module_inception)]\n"
    } else {
        ""
    };
    let mut block: Block = Block::default()
        .with_statement(gen_construction(typing, &type_name))
        .with_statement(EmptyLine::default())
        .with_statement(gen_properties(typing, &type_name, &cases))
        .with_statement(EmptyLine::default())
        .with_statement(gen_cases(typing, &type_name, &cases));
    if cases.iter().any(|case| case.is_nested(typing)) {
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_accessors(typing, &type_name, &cases));
    }
    let schema_name: String = variant.type_name().to_string();
    let schema_name: &str = &schema_name;
    let idents: Vec<String> = cases.iter().map(|case| case.ident()).collect();
    block.add_statement(EmptyLine::default());
    block.add_literal(gen_variant_ord(
        Prelude::new(&type_name),
        &type_name,
        &idents,
    ));
    block.add_statement(EmptyLine::default());
    block.add_literal(gen_native_eq(
        Prelude::new(&type_name),
        &type_name,
        &ref_name(schema_name),
        &buf_name(schema_name),
    ));
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_encoding(typing, &type_name, &cases));
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_decoding(
        typing,
        &type_name,
        &ref_name(schema_name),
        "variant",
    ));
    block.add_statement(EmptyLine::default());
    block.add_literal(gen_element(
        Prelude::new(&type_name),
        &type_name,
        &ref_name(schema_name),
        None,
        1,
        true,
        true,
    ));
    block.add_statement(EmptyLine::default());
    block.add_literal(gen_json(typing, schema_name, &type_name, &cases));
    for name in [
        ref_name(schema_name),
        buf_name(schema_name),
        mut_name(schema_name),
        case_name(schema_name),
        view_name(schema_name),
        view_mut_name(schema_name),
    ] {
        check_type_name(project, mod_path, &name)?;
    }
    let view_mut_cases: Vec<Case> = cases
        .iter()
        .copied()
        .filter(|case| !case.tag.is_collection())
        .collect();
    Ok(vec![
        TypeFile {
            suffix: "",
            code: format!(
                "{}\n{}{}\n{}",
                gen_dec(typing, variant, &type_name).to_code(),
                inception,
                UNRECOGNIZED_MOD,
                block.to_code()
            ),
        },
        TypeFile {
            suffix: "_buf",
            code: gen_buf(
                schema_name,
                &type_name,
                gen_variant_buf_accessors(typing, &cases, &buf_name(schema_name)),
            ),
        },
        TypeFile {
            suffix: "_case",
            code: gen_variant_case(typing, schema_name, &cases, &type_name),
        },
        TypeFile {
            suffix: "_mut",
            code: gen_variant_mut(typing, schema_name, &cases, &type_name),
        },
        TypeFile {
            suffix: "_ref",
            code: gen_variant_ref(typing, schema_name, &cases, &type_name),
        },
        TypeFile {
            suffix: "_view",
            code: gen_variant_view(typing, schema_name, &cases, &type_name),
        },
        TypeFile {
            suffix: "_view_mut",
            code: gen_variant_view_mut(typing, schema_name, &view_mut_cases, &type_name),
        },
    ])
}

/// Checks no two generated methods of the variant with the `type_name` have the same name.
///
/// (ex: the getter `as_bytes` of the case `Bytes`, or the cases `FooBar` & `Foo_Bar`)
fn check_case_names(
    typing: Typing<'_>,
    type_name: &str,
    cases: &[Case<'_>],
) -> Result<(), GenError> {
    let mut names: BTreeSet<String> = ["as_bytes", "as_ref", "as_mut"]
        .into_iter()
        .map(str::to_string)
        .collect();
    for case in cases {
        let mut methods: Vec<String> = vec![case.getter_name()];
        if case.is_nested(typing) {
            methods.push(case.accessor_name());
        } else {
            methods.push(case.setter_name());
        }
        for method in methods {
            if !names.insert(method) {
                return Err(GenError::CaseNameConflict {
                    type_name: type_name.to_string(),
                    case_name: case.name.to_string(),
                });
            }
        }
    }
    Ok(())
}

/// Generates the declaration of the `variant`.
fn gen_dec(typing: Typing<'_>, variant: &Variant, type_name: &str) -> RustEnum {
    let mut result: RustEnum = RustEnum::from(type_name);
    add_docs(&mut result, variant.comments());
    add_derives(&mut result, false, false, false);
    let is_camel_case: bool = is_upper_camel_case(&variant.type_name())
        && variant
            .cases()
            .iter()
            .all(|case| is_upper_camel_case(&case.case_name()));
    if !is_camel_case {
        result.add_attribute("allow(non_camel_case_types)");
    }
    result.set_access(Access::Public);
    result.add_case(
        EnumCase::from("Unrecognized")
            .with_comment(" A case the variant doesn't know.")
            .with_fields(EnumFields::Unnamed(vec![
                "unrecognized::UnrecognizedCase".into(),
            ])),
    );
    for case in variant.cases() {
        let mut case_type: RustType = typing.rust_type(case.type_tag());
        if case.is_boxed() {
            case_type = RustType::from(typing.prelude(PreludeName::Box)).with_generic(case_type);
        }
        let fields: EnumFields = EnumFields::Unnamed(vec![case_type]);
        let mut rust_case: EnumCase = EnumCase::from(ident(&case.case_name())).with_fields(fields);
        add_docs(&mut rust_case, case.comments());
        result.add_case(rust_case);
    }
    result
}

/// Generates the impl block creating an unrecognized case of the variant with the `type_name`.
///
/// Only the variant's own folder can use it, to decode the case.
fn gen_construction(typing: Typing<'_>, type_name: &str) -> ImplBlock {
    let signature: Signature = Signature::from("from_unknown")
        .with_param(Var::from(("tag_number", "u16")))
        .with_param(Var::from((
            "bytes",
            format!("{}<u8>", typing.prelude(PreludeName::Vec)).as_str(),
        )))
        .with_result("Self");
    let from_unknown: Function = Function::from(signature)
        .with_comment(" Creates an unrecognized case.")
        .with_attribute("must_use")
        .with_access(Access::PublicInSuper)
        .with_literal("Self::Unrecognized(unrecognized::UnrecognizedCase { tag_number, bytes })");
    ImplBlock::from(type_name)
        .with_comment(" Construction")
        .with_function(from_unknown)
}

/// Generates the impl block with the properties of the variant with the `type_name`.
fn gen_properties(typing: Typing<'_>, type_name: &str, cases: &[Case<'_>]) -> ImplBlock {
    let signature: Signature = Signature::from("unknown_bytes")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("{}<&[u8]>", typing.prelude(PreludeName::Option)));
    let unknown_bytes: Function = Function::from(signature)
        .with_comment(" Gets the encoded value of an unrecognized case.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "match self {{ Self::Unrecognized(case) => Some(case.bytes.as_slice()), {} }}",
            if cases.is_empty() { "" } else { "_ => None," }
        ));

    let mut arms: Vec<String> = vec!["Self::Unrecognized(case) => case.tag_number,".to_string()];
    for case in cases {
        arms.push(format!("Self::{}(_) => {},", case.ident(), case.number));
    }
    let signature: Signature = Signature::from("tag_number")
        .with_receiver(Receiver::Borrowed)
        .with_result("u16");
    let tag_number: Function = Function::from(signature)
        .with_comment(" Gets the tag number of the case.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_const(true)
        .with_literal(format!("match self {{ {} }}", arms.join(" ")));
    ImplBlock::from(type_name)
        .with_comment(" Properties")
        .with_function(tag_number)
        .with_function(unknown_bytes)
}

/// Generates the impl block with the getter of each of the `cases`.
fn gen_cases(typing: Typing<'_>, type_name: &str, cases: &[Case<'_>]) -> ImplBlock {
    let mut result: ImplBlock = ImplBlock::from(type_name).with_comment(" Cases");
    for case in cases {
        result.add_function(gen_getter(typing, *case));
    }
    result
}

/// Generates the getter of the `case`. (boxes are hidden)
fn gen_getter(typing: Typing<'_>, case: Case<'_>) -> Function {
    let value_type: RustType = typing.rust_type(case.tag);
    let (result, value): (String, String) = if case.tag.is_string() {
        ("&str".to_string(), "value.as_str()".to_string())
    } else if let TypeTag::List(element) = case.tag {
        let method: &str = if case.tag.is_bytes() {
            "as_bytes"
        } else {
            "as_slice"
        };
        (
            format!("&[{}]", typing.rust_type(element)),
            format!("value.{}()", method),
        )
    } else if case.is_nested(typing) || case.tag.is_map() {
        let value: &str = if case.is_boxed { "&**value" } else { "value" };
        (format!("&{}", value_type), value.to_string())
    } else {
        let value: &str = if case.is_boxed { "**value" } else { "*value" };
        (value_type.to_string(), value.to_string())
    };
    let signature: Signature = Signature::from(case.getter_name())
        .with_receiver(Receiver::Borrowed)
        .with_result(format!(
            "{}<{}>",
            typing.prelude(PreludeName::Option),
            result
        ));
    Function::from(signature)
        .with_comment(format!(" Gets the value of the case: `{}`.", case.name))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "match self {{ Self::{}(value) => Some({}), _ => None, }}",
            case.ident(),
            value
        ))
}

/// Generates the impl block with the mutable accessor of each nested case of the `cases`.
fn gen_accessors(typing: Typing<'_>, type_name: &str, cases: &[Case<'_>]) -> ImplBlock {
    let mut result: ImplBlock = ImplBlock::from(type_name).with_comment(" Mutable Cases");
    for case in cases {
        if case.is_nested(typing) {
            result.add_function(gen_accessor(typing, *case));
        }
    }
    result
}

/// Generates the mutable accessor of the nested `case`. (boxes are hidden)
fn gen_accessor(typing: Typing<'_>, case: Case<'_>) -> Function {
    let signature: Signature = Signature::from(case.accessor_name())
        .with_receiver(Receiver::BorrowedMut)
        .with_result(format!(
            "{}<&mut {}>",
            typing.prelude(PreludeName::Option),
            typing.rust_type(case.tag)
        ));
    let value: &str = if case.is_boxed {
        "&mut **value"
    } else {
        "value"
    };
    Function::from(signature)
        .with_comment(format!(
            " Gets the mutable value of the case: `{}`.",
            case.name
        ))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "match self {{ Self::{}(value) => Some({}), _ => None, }}",
            case.ident(),
            value
        ))
}

/// Generates the impl block encoding the variant with the `type_name`.
fn gen_encoding(typing: Typing<'_>, type_name: &str, cases: &[Case<'_>]) -> ImplBlock {
    let mut arms: Vec<String> =
        vec!["Self::Unrecognized(case) => 2 + case.bytes.len(),".to_string()];
    for case in cases {
        arms.push(match case.size {
            Some(size) => format!("Self::{}(_) => {},", case.ident(), 2 + size),
            None if case.tag.is_string() || case.tag.is_bytes() => {
                format!("Self::{}(value) => 2 + value.len(),", case.ident())
            }
            None if case.tag.is_collection() => format!(
                "Self::{}(value) => 2 + {}::{}_encoded_len(value),",
                case.ident(),
                IO,
                collection(case.tag)
            ),
            None => format!("Self::{}(value) => 2 + value.encoded_len(),", case.ident()),
        });
    }
    let signature: Signature = Signature::from("encoded_len")
        .with_receiver(Receiver::Borrowed)
        .with_result("usize");
    let encoded_len: Function = Function::from(signature)
        .with_comment(" Gets the length of the encoding, in bytes.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!("match self {{ {} }}", arms.join(" ")));

    let mut arms: Vec<String> = vec![
        "Self::Unrecognized(case) => { \
            out.extend_from_slice(&case.tag_number.to_le_bytes()); \
            out.extend_from_slice(&case.bytes); \
        }"
        .to_string(),
    ];
    for case in cases {
        arms.push(format!(
            "Self::{}(value) => {{ out.extend_from_slice(&{}_u16.to_le_bytes()); {} }}",
            case.ident(),
            case.number,
            gen_value_write(typing, case.tag, "out")
        ));
    }
    let encode_to: Function = gen_encode_to_with_depth(typing, "variant")
        .with_literal(format!("match self {{ {} }}", arms.join(" ")));

    ImplBlock::from(type_name)
        .with_comment(" Encoding")
        .with_function(encoded_len)
        .with_function(gen_encode(typing, "variant"))
        .with_function(gen_encode_to_of_level(typing, "variant"))
        .with_function(encode_to)
}

/// Generates the serde impls of the variant with the `schema_name`.
///
/// A case is its schema name & value. (ex: `Self_` is `Self`) An unrecognized case is its tag number
/// & its encoded value as base-64 text, which a reader that knows the case decodes as that case.
fn gen_json(typing: Typing<'_>, schema_name: &str, type_name: &str, cases: &[Case<'_>]) -> String {
    let result: &str = typing.prelude(PreludeName::Result);

    let mut serialize: Vec<String> = vec![format!(
        "Self::Unrecognized(case) => {}::serialize_unknown_case(serializer, \"{}\", case.tag_number, &case.bytes),",
        TYPES, schema_name
    )];
    let mut from_case: Vec<String> = Vec::default();
    for case in cases {
        let (written, read): (String, String) = if typing.has_entries_map(case.tag) {
            (
                format!("&{}::JsonRef::new(value)", TYPES),
                format!(
                    "<{}::JsonOwned<{}> as {}::Deserialize>::deserialize(value)?.into_value()",
                    TYPES,
                    typing.rust_type(case.tag),
                    SERDE
                ),
            )
        } else {
            (
                "value".to_string(),
                format!("{}::Deserialize::deserialize(value)?", SERDE),
            )
        };
        serialize.push(format!(
            "Self::{}(value) => {}::serialize_case(serializer, \"{}\", \"{}\", {}),",
            case.ident(),
            TYPES,
            schema_name,
            case.name,
            written
        ));
        from_case.push(format!(
            "\"{}\" => Ok(Self::{}({})),",
            case.name,
            case.ident(),
            read
        ));
    }
    let names: Vec<String> = cases
        .iter()
        .map(|case| format!("\"{}\"", case.name))
        .collect();
    let unknown: String = format!(
        "Err({}::de::Error::unknown_variant(name, &[{}]))",
        SERDE,
        names.join(", ")
    );
    let (value, from_case): (&str, String) = if cases.is_empty() {
        ("_value", unknown)
    } else {
        from_case.push(format!("_ => {},", unknown));
        ("value", format!("match name {{ {} }}", from_case.join(" ")))
    };

    let serialize: String = format!("match self {{ {} }}", serialize.join(" "));
    format!(
        "{serde_impls}
impl {types}::JsonVariant for {type_name} {{
    const TYPE_NAME: &'static str = \"{schema_name}\";

    fn from_case<'de, D>(name: &str, {value}: D) -> {result}<Self, D::Error>
    where
        D: {serde}::Deserializer<'de>,
    {{
        {from_case}
    }}

    fn from_encoded(bytes: &[u8]) -> {result}<Self, {io}::DecodeError> {{
        Self::decode(bytes)
    }}
}}
",
        serde_impls = gen_serde_impls(result, type_name, &serialize, "deserialize_variant"),
        serde = SERDE,
        types = TYPES,
        io = IO,
        type_name = type_name,
        schema_name = schema_name,
        result = result,
        value = value,
        from_case = from_case,
    )
}
