use crate::GenError;
use crate::rust::derives::add_derives;
use crate::rust::docs::{PpsLine, add_pps_docs};
use crate::rust::fields::{Field, check_field_names};
use crate::rust::naming::{ident, is_upper_camel_case};
use crate::rust::typing::Typing;
use code_gen::rust::{Access, Struct as RustStruct, WithAccess, WithAttributes, WithStructFields};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{
    Message, ModPathRef, Project, WithComments as _, WithFieldName, WithTypeName, WithTypeTag,
};

/// The name of the constructor. (from `Default`)
const CONSTRUCTOR: &str = "default";

/// Generates the Rust code of the `message` in the schema file at the `mod_path`.
///
/// Every message field is optional, so messages are created with `Default`. No field can be named
/// like a generated method. (ex: the constructor `default`)
pub fn gen_message(
    project: &Project,
    mod_path: ModPathRef<'_>,
    message: &Message,
) -> Result<String, GenError> {
    let typing: Typing = Typing::new(project);
    let type_name: String = ident(&message.type_name());
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
        &[CONSTRUCTOR],
    )?;

    let mut block: Block = Block::default().with_statement(gen_dec(typing, message, &fields));
    for field in &fields {
        block.add_statement(EmptyLine::default());
        block.add_statement(field.gen_impl(typing, &type_name));
    }
    Ok(block.to_code())
}

/// Generates the declaration of the `message`.
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
    add_derives(&mut result, false, true);
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
    result
}

#[cfg(test)]
mod tests {
    use crate::GenError;
    use crate::rust::gen_message;
    use proto_packet::io::TagNumber;
    use proto_packet_tree::{
        FieldName, Message, MessageField, ModPath, PrimitiveType, Project, TypeName,
    };

    #[test]
    fn gen_message_rejects_the_constructor_name() {
        let mod_path: ModPath = ModPath::try_from("a").unwrap();
        let field: MessageField = MessageField::new(
            FieldName::try_from("default").unwrap(),
            PrimitiveType::UnsignedInt32,
            TagNumber::new(1).unwrap(),
        );
        let message: Message = Message::from(TypeName::try_from("Foo").unwrap())
            .with_field(field)
            .unwrap();
        let result: Result<String, GenError> =
            gen_message(&Project::default(), mod_path.to_ref(), &message);
        let expected: GenError = GenError::FieldNameConflict {
            type_name: "a.Foo".to_string(),
            field_name: "default".to_string(),
        };
        assert_eq!(result, Err(expected));
    }
}
