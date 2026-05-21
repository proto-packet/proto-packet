use crate::validate_name;
use custom_string::custom_string;

custom_string!(
    #[doc = "The name of a struct or message field."],
    FieldName,
    validate_field_name
);

/// Validates the `field_name`.
pub fn validate_field_name(field_name: &str) -> Result<(), &'static str> {
    validate_name(field_name)?;

    if !field_name.as_bytes()[0].is_ascii_lowercase() {
        Err("field names must start with a lowercase letter")
    } else {
        Ok(())
    }
}
