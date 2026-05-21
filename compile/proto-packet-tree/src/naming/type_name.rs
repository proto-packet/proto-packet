use crate::validate_name;
use custom_string::custom_string;

custom_string!(
    #[doc = "The name of a type."],
    TypeName,
    validate_type_name
);

/// Validates the `type_name`.
pub fn validate_type_name(type_name: &str) -> Result<(), &'static str> {
    validate_name(type_name)?;

    if !type_name.as_bytes()[0].is_ascii_uppercase() {
        Err("type names must start with an uppercase letter")
    } else {
        Ok(())
    }
}
