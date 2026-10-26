use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::{ValidateError, validate_tag_number, validate_type_tag};
use proto_packet::io::TagNumber;
use proto_packet_parse::MessageFieldTree;
use proto_packet_tree::{FieldName, MessageField, TypeTag};

/// Validates the message field `tree`.
pub fn validate_message_field(
    tree: &MessageFieldTree,
    source: &str,
) -> Result<MessageField, ValidateError> {
    let field_name: FieldName = validate_name(tree.field_name, source)?;
    let type_tag: TypeTag = validate_type_tag(&tree.type_tag, source)?;
    let tag_number: TagNumber = validate_tag_number(tree.tag_number, source)?;
    let mut field: MessageField = MessageField::new(field_name, type_tag, tag_number);
    add_comments(&mut field, &tree.comments, source);
    Ok(field)
}
