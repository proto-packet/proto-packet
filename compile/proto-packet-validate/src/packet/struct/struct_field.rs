use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::var::validate_boxed;
use crate::{ValidateError, validate_type_tag};
use proto_packet_parse::StructFieldTree;
use proto_packet_tree::{FieldName, StructField, TypeTag};

/// Validates the struct field `tree`.
pub fn validate_struct_field(
    tree: &StructFieldTree,
    source: &str,
) -> Result<StructField, ValidateError> {
    let field_name: FieldName = validate_name(tree.field_name, source)?;
    let type_tag: TypeTag = validate_type_tag(&tree.type_tag, source)?;
    validate_boxed(tree.is_boxed, &type_tag, tree.field_name)?;
    let mut field: StructField = StructField::new(field_name, type_tag)
        .with_optional(tree.is_optional)
        .with_boxed(tree.is_boxed);
    add_comments(&mut field, &tree.comments, source);
    Ok(field)
}
