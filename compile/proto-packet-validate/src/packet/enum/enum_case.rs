use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::{ValidateError, validate_tag_number};
use proto_packet::io::TagNumber;
use proto_packet_parse::EnumCaseTree;
use proto_packet_tree::{CaseName, EnumCase};

/// Validates the enum case `tree`.
pub fn validate_enum_case(tree: &EnumCaseTree, source: &str) -> Result<EnumCase, ValidateError> {
    let case_name: CaseName = validate_name(tree.case_name, source)?;
    let tag_number: TagNumber = validate_tag_number(tree.tag_number, source)?;
    let mut case: EnumCase = EnumCase::new(case_name, tag_number);
    add_comments(&mut case, &tree.comments, source);
    Ok(case)
}
