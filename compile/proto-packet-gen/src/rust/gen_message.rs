use crate::GenError;
use crate::layout::MessageLayout;
use crate::rust::TypeFile;
use crate::rust::derives::add_derives;
use crate::rust::docs::{PpsLine, add_pps_docs};
use crate::rust::encoding::IO;
use crate::rust::fields::{Field, check_field_names};
use crate::rust::gen_buf::{buf_name, gen_buf};
use crate::rust::gen_element::gen_element;
use crate::rust::gen_json::{SERDE, TYPES, gen_serde_impls};
use crate::rust::gen_message_mut::{gen_message_buf_accessors, gen_message_mut};
use crate::rust::gen_message_ref::gen_message_ref;
use crate::rust::gen_ord::gen_message_ord;
use crate::rust::gen_struct::{
    check_list_elements, check_type_name, gen_accessor, gen_decoding, view_fields,
};
use crate::rust::gen_struct_mut::{check_mut_names, is_struct, mut_name};
use crate::rust::gen_struct_ref::ref_name;
use crate::rust::gen_view::{gen_view, view_name};
use crate::rust::gen_view_mut::{gen_view_mut, view_mut_name};
use crate::rust::message_encoding::{TaggedField, gen_message_encoding};
use crate::rust::naming::{ident, is_upper_camel_case};
use crate::rust::prelude::{Prelude, PreludeName};
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, Signature, Struct as RustStruct, StructField, Var,
    WithAccess, WithAttributes, WithComments, WithFunctions, WithReceiver, WithResult,
    WithStructFields, WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{
    Message, ModPathRef, Project, WithComments as _, WithFieldName, WithTypeName, WithTypeTag,
};

/// The name of the constructor. (from `Default`)
const CONSTRUCTOR: &str = "default";

/// The name of the unknown fields.
const UNKNOWN_FIELDS: &str = "unknown_fields";

/// The type of an unknown field.
const UNKNOWN_FIELD: &str = "proto_packet::io::UnknownField";

/// The names of the generated methods of a message & its `Ref` type.
const METHODS: [&str; 12] = [
    CONSTRUCTOR,
    UNKNOWN_FIELDS,
    "with_unknown_fields",
    "encoded_len",
    "encode",
    "encode_to",
    "decode",
    "decode_unchecked",
    "new",
    "new_unchecked",
    "as_bytes",
    "to_buf",
];

/// Generates the Rust code of the `message` in the schema file at the `mod_path`.
///
/// Every message field is optional, so messages are created with `Default`. No field can be named
/// like a generated method. (ex: the constructor `default`)
pub fn gen_message(
    project: &Project,
    mod_path: ModPathRef<'_>,
    message: &Message,
) -> Result<Vec<TypeFile>, GenError> {
    let type_name: String = ident(&message.type_name());
    let typing: Typing = Typing::new(project, Prelude::new(&type_name));
    let fields: Vec<Field> = message
        .fields()
        .iter()
        .map(|field| Field {
            name: field.field_name(),
            tag: field.type_tag(),
            is_optional: true,
            is_boxed: field.is_boxed(),
        })
        .collect();
    check_list_elements(
        project,
        &format!("{}.{}", mod_path, message.type_name()),
        fields.iter().map(|field| (field.name.value(), field.tag)),
    )?;
    let layout: MessageLayout = MessageLayout::new(project, message);
    let accessors: Vec<String> = fields
        .iter()
        .filter(|field| is_struct(typing, **field))
        .map(|field| field.accessor_name())
        .collect();
    let methods: Vec<&str> = METHODS
        .into_iter()
        .chain(accessors.iter().map(String::as_str))
        .collect();
    check_field_names(
        &format!("{}.{}", mod_path, message.type_name()),
        &fields,
        &methods,
    )?;

    let mut block: Block = Block::default().with_statement(gen_dec(typing, message, &fields));
    for field in &fields {
        let mut field_impl: ImplBlock = field.gen_impl(typing, &type_name);
        if is_struct(typing, *field) {
            field_impl.add_function(gen_accessor(typing, *field));
        }
        block.add_statement(EmptyLine::default());
        block.add_statement(field_impl);
    }
    let tagged: Vec<TaggedField> = layout
        .fields
        .iter()
        .map(|field| TaggedField {
            field: fields[field.index],
            tag: field.tag,
            size: field.size,
        })
        .collect();
    let schema_name: String = message.type_name().to_string();
    let schema_name: &str = &schema_name;
    let ordered: Vec<String> = tagged
        .iter()
        .map(|tagged| ident(&tagged.field.name))
        .collect();
    block.add_statement(EmptyLine::default());
    block.add_literal(gen_message_ord(
        Prelude::new(&type_name),
        &type_name,
        &ordered,
    ));
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_unknown_fields(typing, &type_name));
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_message_encoding(typing, &type_name, &tagged));
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_decoding(
        typing,
        &type_name,
        &ref_name(schema_name),
        "message",
    ));
    block.add_statement(EmptyLine::default());
    block.add_literal(gen_element(
        Prelude::new(&type_name),
        &type_name,
        &ref_name(schema_name),
        None,
        1,
        true,
    ));
    block.add_statement(EmptyLine::default());
    block.add_literal(gen_json(typing, schema_name, &type_name, &fields));
    check_type_name(project, mod_path, &ref_name(schema_name))?;
    check_type_name(project, mod_path, &buf_name(schema_name))?;
    check_type_name(project, mod_path, &mut_name(schema_name))?;
    check_type_name(project, mod_path, &view_name(schema_name))?;
    check_type_name(project, mod_path, &view_mut_name(schema_name))?;
    check_mut_names(typing, &format!("{}.{}", mod_path, schema_name), &fields)?;
    Ok(vec![
        TypeFile {
            suffix: "",
            code: block.to_code(),
        },
        TypeFile {
            suffix: "_buf",
            code: gen_buf(
                schema_name,
                &type_name,
                gen_message_buf_accessors(typing, &tagged, &buf_name(schema_name)),
            ),
        },
        TypeFile {
            suffix: "_mut",
            code: gen_message_mut(typing, schema_name, &tagged, &type_name),
        },
        TypeFile {
            suffix: "_ref",
            code: gen_message_ref(typing, schema_name, &tagged, &type_name),
        },
        TypeFile {
            suffix: "_view",
            code: gen_view(typing, schema_name, &view_fields(&fields), &type_name),
        },
        TypeFile {
            suffix: "_view_mut",
            code: gen_view_mut(typing, schema_name, &view_fields(&fields), &type_name, true),
        },
    ])
}

/// Generates the impl block with the unknown fields of the message with the `type_name`.
fn gen_unknown_fields(typing: Typing<'_>, type_name: &str) -> ImplBlock {
    let unknown_type: String = format!("{}<{}>", typing.prelude(PreludeName::Vec), UNKNOWN_FIELD);
    let signature: Signature = Signature::from("unknown_fields")
        .with_receiver(Receiver::Borrowed)
        .with_result(format!("&[{}]", UNKNOWN_FIELD));
    let unknown_fields: Function = Function::from(signature)
        .with_comment(" Gets the fields the message doesn't know, in tag order.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal("&self.unknown_fields");
    let signature: Signature = Signature::from("with_unknown_fields")
        .with_receiver(Receiver::OwnedMut)
        .with_param(Var::from(("fields", unknown_type.as_str())))
        .with_result("Self");
    let with_unknown_fields: Function = Function::from(signature)
        .with_comment(" Sets the unknown fields. (builder pattern)")
        .with_attribute("must_use")
        .with_access(Access::PublicInSuper)
        .with_semi("self.unknown_fields = fields")
        .with_literal("self");
    ImplBlock::from(type_name)
        .with_comment(" Unknown Fields")
        .with_function(unknown_fields)
        .with_function(with_unknown_fields)
}

/// Generates the declaration of the `message`.
///
/// A message keeps its unknown fields, so they survive decoding & encoding.
fn gen_dec(typing: Typing<'_>, message: &Message, fields: &[Field<'_>]) -> RustStruct {
    let mut result: RustStruct = RustStruct::from(ident(&message.type_name()));
    let lines: Vec<PpsLine> = message
        .fields()
        .iter()
        .map(|field| PpsLine {
            comments: field.comments(),
            line: format!(
                "{}: {}{} = {};",
                field.field_name(),
                if field.is_boxed() { "box " } else { "" },
                field.type_tag(),
                field.tag_number()
            ),
        })
        .collect();
    add_pps_docs(
        &mut result,
        "message",
        message.type_name().value(),
        message.comments(),
        &lines,
    );
    add_derives(&mut result, false, false, true);
    if !is_upper_camel_case(&message.type_name()) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    if fields.iter().any(|field| field.is_non_snake_case()) {
        result.add_attribute("allow(non_snake_case)");
    }
    result.set_access(Access::Public);
    for field in fields {
        result.add_field(field.gen_plain_dec(typing));
    }
    let unknown_type: String = format!("{}<{}>", typing.prelude(PreludeName::Vec), UNKNOWN_FIELD);
    result.add_field(StructField::from((UNKNOWN_FIELDS, unknown_type.as_str())));
    result
}

/// Generates the serde impls of the message with the `schema_name` & the `fields`.
///
/// A field is its schema name & value. (ex: `self_` is `self`) An unknown field is its tag number &
/// its encoded value as base-64 text, which a reader that knows the field decodes as that field.
fn gen_json(
    typing: Typing<'_>,
    schema_name: &str,
    type_name: &str,
    fields: &[Field<'_>],
) -> String {
    let result: &str = typing.prelude(PreludeName::Result);

    let mut serialize: Vec<String> = Vec::default();
    let mut read_field: Vec<String> = Vec::default();
    let mut merge: Vec<String> = Vec::default();
    for field in fields {
        let name: String = ident(&field.name);
        serialize.push(format!(
            "if let Some(value) = &self.{} {{ {}::ser::SerializeMap::serialize_entry(&mut map, \"{}\", value)?; }}",
            name, SERDE, field.name
        ));
        read_field.push(format!(
            "\"{}\" => {{ self.{} = map.next_value()?; Ok(true) }}",
            field.name, name
        ));
        merge.push(format!(
            "if other.{name}.is_some() {{ if self.{name}.is_some() {{ return Err(\"{key}\"); }} self.{name} = other.{name}; }}",
            key = field.name,
            name = name
        ));
    }
    let (name_param, map_param, read_body): (&str, &str, String) = if fields.is_empty() {
        ("_name", "_map", "Ok(false)".to_string())
    } else {
        read_field.push("_ => Ok(false),".to_string());
        (
            "name",
            "map",
            format!("match name {{ {} }}", read_field.join(" ")),
        )
    };

    let serialize: String = format!(
        "let mut map: S::SerializeMap = {serde}::Serializer::serialize_map(serializer, None)?;
        {fields}
        {types}::serialize_unknown_fields(&mut map, &self.unknown_fields)?;
        {serde}::ser::SerializeMap::end(map)",
        serde = SERDE,
        types = TYPES,
        fields = serialize.join(" "),
    );
    format!(
        "{serde_impls}
impl {types}::JsonMessage for {type_name} {{
    const TYPE_NAME: &'static str = \"{schema_name}\";

    fn read_field<'de, A>(&mut self, {name_param}: &str, {map_param}: &mut A) -> {result}<bool, A::Error>
    where
        A: {serde}::de::MapAccess<'de>,
    {{
        {read_body}
    }}

    fn from_encoded(bytes: &[u8]) -> {result}<Self, {io}::DecodeError> {{
        Self::decode(bytes)
    }}

    fn merge(&mut self, other: Self) -> {result}<(), &'static str> {{
        {merge}
        self.unknown_fields = other.unknown_fields;
        Ok(())
    }}
}}
",
        serde_impls = gen_serde_impls(result, type_name, &serialize, "deserialize_message"),
        serde = SERDE,
        types = TYPES,
        io = IO,
        type_name = type_name,
        schema_name = schema_name,
        result = result,
        name_param = name_param,
        map_param = map_param,
        read_body = read_body,
        merge = merge.join(" "),
    )
}
