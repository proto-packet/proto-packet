use crate::{ValidateError, validate_type_tag};
use proto_packet_parse::StructFieldTree;
use proto_packet_tree::{FieldName, StructField, TypeTag, WithComments};

/// Validates the struct field `tree`.
pub fn validate_struct_field(
    tree: &StructFieldTree,
    source: &str,
) -> Result<StructField, ValidateError> {
    let field_name: FieldName =
        FieldName::new(tree.field_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.field_name,
            message: e.message(),
        })?;
    let type_tag: TypeTag = validate_type_tag(&tree.type_tag, source)?;

    let mut field: StructField = StructField::new(field_name, type_tag);
    field.set_optional(tree.is_optional);
    for comment in &tree.comments {
        field.add_comment(comment.text(source));
    }

    Ok(field)
}
