use crate::validate_name;
use custom_string::custom_string;

custom_string!(
    #[doc = "The name of a service call."],
    CallName,
    validate_call_name
);

/// Validates the `call_name`.
pub fn validate_call_name(call_name: &str) -> Result<(), &'static str> {
    validate_name(call_name)?;

    if !call_name.as_bytes()[0].is_ascii_lowercase() {
        Err("call names must start with a lowercase letter")
    } else {
        Ok(())
    }
}
