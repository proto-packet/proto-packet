use crate::{ValidateError, validate_type_tag};
use proto_packet::io::TagNumber;
use proto_packet_parse::MessageFieldTree;
use proto_packet_tree::{FieldName, MessageField, TypeTag, WithComments};

/// Validates the message field `tree`.
pub fn validate_message_field(
    tree: &MessageFieldTree,
    source: &str,
) -> Result<MessageField, ValidateError> {
    let field_name: FieldName =
        FieldName::new(tree.field_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.field_name,
            message: e.message(),
        })?;
    let type_tag: TypeTag = validate_type_tag(&tree.type_tag, source)?;
    let tag_number: TagNumber = tree
        .tag_number
        .text(source)
        .parse::<u16>()
        .ok()
        .and_then(TagNumber::new)
        .ok_or(ValidateError::InvalidTagNumber {
            span: tree.tag_number,
            message: "tag number must be in 1..=8191",
        })?;

    let mut field: MessageField = MessageField::new(field_name, type_tag, tag_number);
    for comment in &tree.comments {
        field.add_comment(comment.text(source));
    }

    Ok(field)
}
