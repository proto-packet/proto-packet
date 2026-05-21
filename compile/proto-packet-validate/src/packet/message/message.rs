use crate::ValidateError;
use crate::packet::validate_message_field;
use proto_packet_parse::MessageTree;
use proto_packet_tree::{Message, MessageField, TypeName, WithComments};

/// Validates the message `tree`.
pub fn validate_message(tree: &MessageTree, source: &str) -> Result<Message, ValidateError> {
    let type_name: TypeName =
        TypeName::new(tree.type_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.type_name,
            message: e.message(),
        })?;

    let mut message: Message = Message::from(type_name);
    for comment in &tree.comments {
        message.add_comment(comment.text(source));
    }

    for field_tree in &tree.fields {
        let field: MessageField = validate_message_field(field_tree, source)?;
        message
            .add_field(field)
            .map_err(|error| ValidateError::Tree { error })?;
    }

    Ok(message)
}
