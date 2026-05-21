use crate::ValidateError;
use crate::packet::validate_struct_field;
use proto_packet_parse::StructTree;
use proto_packet_tree::{Struct, StructField, TypeName, WithComments};

/// Validates the struct `tree`.
pub fn validate_struct(tree: &StructTree, source: &str) -> Result<Struct, ValidateError> {
    let type_name: TypeName =
        TypeName::new(tree.type_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.type_name,
            message: e.message(),
        })?;

    let mut structure: Struct = Struct::from(type_name);
    for comment in &tree.comments {
        structure.add_comment(comment.text(source));
    }

    for field_tree in &tree.fields {
        let field: StructField = validate_struct_field(field_tree, source)?;
        structure
            .add_field(field)
            .map_err(|error| ValidateError::Tree { error })?;
    }

    Ok(structure)
}
