use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::var::tagged_member_error;
use crate::{ValidateError, validate_enum_case};
use proto_packet_parse::EnumTree;
use proto_packet_tree::{Enum, EnumCase, TypeName};

/// Validates the enum `tree`.
pub fn validate_enum(tree: &EnumTree, source: &str) -> Result<Enum, ValidateError> {
    let type_name: TypeName = validate_name(tree.type_name, source)?;
    let mut enumeration: Enum = Enum::from(type_name);
    add_comments(&mut enumeration, &tree.comments, source);
    for case_tree in &tree.cases {
        let case: EnumCase = validate_enum_case(case_tree, source)?;
        enumeration.add_case(case).map_err(|error| {
            tagged_member_error(error, case_tree.case_name, case_tree.tag_number)
        })?;
    }
    Ok(enumeration)
}
