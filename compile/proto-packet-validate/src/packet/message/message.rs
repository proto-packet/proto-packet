use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::{ValidateError, ValidateErrorReason, validate_message_field};
use lex::lexer::Span;
use proto_packet_parse::MessageTree;
use proto_packet_tree::{Message, MessageField, TreeError, TypeName};

/// Validates the message `tree`.
pub fn validate_message(tree: &MessageTree, source: &str) -> Result<Message, ValidateError> {
    let type_name: TypeName = validate_name(tree.type_name, source)?;
    let mut message: Message = Message::from(type_name);
    add_comments(&mut message, &tree.comments, source);
    for field_tree in &tree.fields {
        let field: MessageField = validate_message_field(field_tree, source)?;
        message.add_field(field).map_err(|error| {
            let span: Span = match error {
                TreeError::DuplicateTagNumber { .. } => field_tree.tag_number,
                _ => field_tree.field_name,
            };
            ValidateError::new(span, ValidateErrorReason::Tree(error))
        })?;
    }
    Ok(message)
}
