use crate::validate_name;
use custom_string::custom_string;

custom_string!(
    #[doc = "The name of a module."],
    ModName,
    validate_mod_name
);

/// Validates the `mod_name`.
pub fn validate_mod_name(mod_name: &str) -> Result<(), &'static str> {
    validate_name(mod_name)?;

    if !mod_name.as_bytes()[0].is_ascii_lowercase() {
        Err("mod names must start with a lowercase letter")
    } else {
        Ok(())
    }
}
