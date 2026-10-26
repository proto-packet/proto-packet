use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::{ValidateError, ValidateErrorReason, validate_struct_field};
use proto_packet_parse::StructTree;
use proto_packet_tree::{Struct, StructField, TypeName};

/// Validates the struct `tree`.
pub fn validate_struct(tree: &StructTree, source: &str) -> Result<Struct, ValidateError> {
    let type_name: TypeName = validate_name(tree.type_name, source)?;
    let mut structure: Struct = Struct::from(type_name);
    add_comments(&mut structure, &tree.comments, source);
    for field_tree in &tree.fields {
        let field: StructField = validate_struct_field(field_tree, source)?;
        structure.add_field(field).map_err(|error| {
            ValidateError::new(field_tree.field_name, ValidateErrorReason::Tree(error))
        })?;
    }
    Ok(structure)
}
