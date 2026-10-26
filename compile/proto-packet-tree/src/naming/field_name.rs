use crate::naming::validate_name::validate_name;
use custom_string::custom_string;

custom_string!(
    #[doc = "The name of a struct or message field. (ex: my_field)"],
    FieldName,
    validate_field_name
);

/// Validates the `field_name`.
fn validate_field_name(field_name: &str) -> Result<(), &'static str> {
    validate_name(field_name)?;

    if !field_name.starts_with(|c: char| c.is_ascii_lowercase()) {
        Err("field names must start with a lowercase letter")
    } else {
        Ok(())
    }
}
