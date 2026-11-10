use enc::base_64::{Base64Decoder, Base64Encoder, Base64Validator};
use enc::{Decoder, Error, StringEncoder, Validator};

/// Encodes the `bytes` as base-64 text.
pub(crate) fn encode_base_64(bytes: &[u8]) -> Result<String, Error> {
    Base64Encoder::default().encode_as_string(bytes)
}

/// Appends the decoded base-64 `text` to the `target`.
///
/// The decoder doesn't validate, so the `text` is validated first.
///
/// # Note
/// - The text has its padding, so its length is a multiple of 4.
pub(crate) fn append_base_64(text: &str, target: &mut Vec<u8>) -> Result<(), String> {
    let is_valid: bool = Base64Validator::default()
        .is_valid(text.as_bytes())
        .map_err(|error| error.to_string())?;
    if !is_valid || !text.len().is_multiple_of(4) {
        return Err("invalid base-64".to_string());
    }
    Base64Decoder::default()
        .append_to_vec(text.as_bytes(), target)
        .map_err(|error| error.to_string())?;
    Ok(())
}
