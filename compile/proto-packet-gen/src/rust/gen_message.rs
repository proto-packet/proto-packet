use crate::GenError;
use crate::layout::MessageLayout;
use crate::rust::TypeFile;
use crate::rust::derives::{add_derives, add_serde_derives};
use crate::rust::docs::{PpsLine, add_pps_docs};
use crate::rust::fields::{Field, check_field_names};
use crate::rust::gen_buf::{buf_name, gen_buf};
use crate::rust::gen_message_mut::{gen_message_buf_accessors, gen_message_mut};
use crate::rust::gen_message_ref::gen_message_ref;
use crate::rust::gen_struct::{check_type_name, gen_decoding};
use crate::rust::gen_struct_mut::{check_mut_names, mut_name};
use crate::rust::gen_struct_ref::ref_name;
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
    check_field_names(
        &format!("{}.{}", mod_path, message.type_name()),
        &fields,
        &METHODS,
    )?;
    let layout: Option<MessageLayout> = MessageLayout::new(project, message);

    let mut block: Block =
        Block::default().with_statement(gen_dec(typing, message, &fields, layout.is_some()));
    for field in &fields {
        block.add_statement(EmptyLine::default());
        block.add_statement(field.gen_impl(typing, &type_name));
    }
    let mut files: Vec<TypeFile> = Vec::default();
    if let Some(layout) = &layout {
        let tagged: Vec<TaggedField> = layout
            .fields
            .iter()
            .map(|field| TaggedField {
                field: fields[field.index],
                tag: field.tag,
                size: field.size,
            })
            .collect();
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_unknown_fields(typing, &type_name));
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_message_encoding(typing, &type_name, &tagged));
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_decoding(
            typing,
            &type_name,
            &ref_name(&message.type_name()),
            "message",
        ));
        check_type_name(project, mod_path, &ref_name(&message.type_name()))?;
        check_type_name(project, mod_path, &buf_name(&message.type_name()))?;
        check_type_name(project, mod_path, &mut_name(&message.type_name()))?;
        check_mut_names(
            typing,
            &format!("{}.{}", mod_path, message.type_name()),
            &fields,
        )?;
        files.push(TypeFile {
            suffix: "_buf",
            code: gen_buf(
                &message.type_name(),
                &type_name,
                gen_message_buf_accessors(typing, &tagged, &buf_name(&message.type_name())),
            ),
        });
        files.push(TypeFile {
            suffix: "_mut",
            code: gen_message_mut(typing, &message.type_name(), &tagged, &type_name),
        });
        files.push(TypeFile {
            suffix: "_ref",
            code: gen_message_ref(typing, &message.type_name(), &tagged, &type_name),
        });
    }
    files.insert(
        0,
        TypeFile {
            suffix: "",
            code: block.to_code(),
        },
    );
    Ok(files)
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
/// A message with an encoding keeps its unknown fields, so they survive decoding & encoding.
fn gen_dec(
    typing: Typing<'_>,
    message: &Message,
    fields: &[Field<'_>],
    has_encoding: bool,
) -> RustStruct {
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
    add_derives(&mut result, false, true, true);
    add_serde_derives(&mut result);
    if !is_upper_camel_case(&message.type_name()) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    if fields.iter().any(|field| field.is_non_snake_case()) {
        result.add_attribute("allow(non_snake_case)");
    }
    result.set_access(Access::Public);
    for field in fields {
        result.add_field(field.gen_dec(typing));
    }
    if has_encoding {
        let unknown_type: String =
            format!("{}<{}>", typing.prelude(PreludeName::Vec), UNKNOWN_FIELD);
        let mut unknown_fields: StructField =
            StructField::from((UNKNOWN_FIELDS, unknown_type.as_str()));
        unknown_fields.add_attribute("serde(skip)");
        result.add_field(unknown_fields);
    }
    result
}
