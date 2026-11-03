use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::{ValidateError, ValidateErrorReason, validate_tag_number};
use proto_packet::io::TagNumber;
use proto_packet_parse::EnumCaseTree;
use proto_packet_tree::{CaseName, EnumCase};

/// The case name reserved for the tag numbers of the cases from newer schemas.
const UNRECOGNIZED: &str = "Unrecognized";

/// Validates the enum case `tree`.
///
/// No case can be named `Unrecognized`, since each language generates that case.
pub fn validate_enum_case(tree: &EnumCaseTree, source: &str) -> Result<EnumCase, ValidateError> {
    let case_name: CaseName = validate_name(tree.case_name, source)?;
    if case_name.value() == UNRECOGNIZED {
        let message: &'static str = "enum cases cannot be named `Unrecognized`";
        return Err(ValidateError::new(
            tree.case_name,
            ValidateErrorReason::InvalidName { message },
        ));
    }
    let tag_number: TagNumber = validate_tag_number(tree.tag_number, source)?;
    let mut case: EnumCase = EnumCase::new(case_name, tag_number);
    add_comments(&mut case, &tree.comments, source);
    Ok(case)
}
