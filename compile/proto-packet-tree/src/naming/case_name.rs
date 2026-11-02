use crate::naming::validate_name::validate_name;
use custom_string::custom_string;

custom_string!(
    #[doc = "The name of an enum or variant case. (ex: MyCase)"],
    CaseName,
    validate_case_name
);

/// Validates the `case_name`.
fn validate_case_name(case_name: &str) -> Result<(), &'static str> {
    validate_name(case_name)?;

    if !case_name.starts_with(|c: char| c.is_ascii_uppercase()) {
        Err("case names must start with an uppercase letter")
    } else {
        Ok(())
    }
}
