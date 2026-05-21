/// Validates the `name`.
///
/// This function only validates the packet properties of name types.
pub fn validate_name(name: &str) -> Result<(), &'static str> {
    let bytes: &[u8] = name.as_bytes();
    Err(if bytes.is_empty() {
        "names cannot be empty"
    } else if bytes
        .iter()
        .any(|c| !c.is_ascii_alphanumeric() && *c != b'_')
    {
        "names must only contain: [a-zA-Z0-9_]"
    } else if bytes[0] == b'_' {
        "names cannot start with an underscore"
    } else if bytes[bytes.len() - 1] == b'_' {
        "names cannot end with an underscore"
    } else if name.contains("__") {
        "names cannot contain a double underscore"
    } else {
        return Ok(());
    })
}
