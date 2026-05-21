use crate::validate_name;
use custom_string::custom_string;

custom_string!(
    #[doc = "The name of an enum or variant case."],
    CaseName,
    validate_case_name
);

/// Validates the `case_name`.
pub fn validate_case_name(case_name: &str) -> Result<(), &'static str> {
    validate_name(case_name)?;

    if !case_name.as_bytes()[0].is_ascii_uppercase() {
        Err("case names must start with an uppercase letter")
    } else {
        Ok(())
    }
}
