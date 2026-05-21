use crate::ValidateError;
use proto_packet::io::TagNumber;
use proto_packet_parse::EnumCaseTree;
use proto_packet_tree::{CaseName, EnumCase, WithComments};

/// Validates the enum case `tree`.
pub fn validate_enum_case(tree: &EnumCaseTree, source: &str) -> Result<EnumCase, ValidateError> {
    let case_name: CaseName =
        CaseName::new(tree.case_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.case_name,
            message: e.message(),
        })?;
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

    let mut case: EnumCase = EnumCase::new(case_name, tag_number);
    for comment in &tree.comments {
        case.add_comment(comment.text(source));
    }

    Ok(case)
}
