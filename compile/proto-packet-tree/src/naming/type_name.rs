use crate::naming::validate_name::validate_name;
use custom_string::custom_string;

custom_string!(
    #[doc = "The name of a type. (ex: MyType)"],
    TypeName,
    validate_type_name
);

/// Validates the `type_name`.
fn validate_type_name(type_name: &str) -> Result<(), &'static str> {
    validate_name(type_name)?;

    if !type_name.starts_with(|c: char| c.is_ascii_uppercase()) {
        Err("type names must start with an uppercase letter")
    } else {
        Ok(())
    }
}
