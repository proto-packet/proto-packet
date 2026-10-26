/// Validates the `name`.
///
/// This only validates the rules shared by every kind of name.
pub(crate) fn validate_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() {
        Err("names cannot be empty")
    } else if !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
        Err("names must only contain: [a-zA-Z0-9_]")
    } else if name.starts_with('_') || name.ends_with('_') {
        Err("names cannot start or end with an underscore")
    } else if name.contains("__") {
        Err("names cannot contain a double underscore")
    } else {
        Ok(())
    }
}
