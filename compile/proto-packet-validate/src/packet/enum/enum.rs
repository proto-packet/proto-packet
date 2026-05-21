use crate::ValidateError;
use crate::packet::validate_enum_case;
use proto_packet_parse::EnumTree;
use proto_packet_tree::{Enum, EnumCase, TypeName, WithComments};

/// Validates the enum `tree`.
pub fn validate_enum(tree: &EnumTree, source: &str) -> Result<Enum, ValidateError> {
    let type_name: TypeName =
        TypeName::new(tree.type_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.type_name,
            message: e.message(),
        })?;

    let mut enumeration: Enum = Enum::from(type_name);
    for comment in &tree.comments {
        enumeration.add_comment(comment.text(source));
    }

    for case_tree in &tree.cases {
        let case: EnumCase = validate_enum_case(case_tree, source)?;
        enumeration
            .add_case(case)
            .map_err(|error| ValidateError::Tree { error })?;
    }

    Ok(enumeration)
}
