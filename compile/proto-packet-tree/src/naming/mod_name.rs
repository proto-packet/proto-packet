use crate::naming::validate_name::validate_name;
use custom_string::custom_string;

custom_string!(
    #[doc = "The name of a module. (ex: my_mod)"],
    ModName,
    validate_mod_name
);

/// Validates the `mod_name`.
fn validate_mod_name(mod_name: &str) -> Result<(), &'static str> {
    validate_name(mod_name)?;

    if !mod_name.starts_with(|c: char| c.is_ascii_lowercase()) {
        Err("mod names must start with a lowercase letter")
    } else {
        Ok(())
    }
}
