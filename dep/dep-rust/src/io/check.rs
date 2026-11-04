use crate::io::{DecodeError, TagNumber};

/// Checks the `bytes` can hold a fixed block of the `size`.
///
/// # Errors
/// Returns an error if the `bytes` are shorter than the `size`.
pub const fn check_len(bytes: &[u8], size: usize) -> Result<(), DecodeError> {
    if bytes.len() < size {
        return Err(DecodeError::TooShort {
            expected: size,
            actual: bytes.len(),
        });
    }
    Ok(())
}

/// Checks the end table at the `table`, with the `count` of entries, for the tail at the `tail`.
///
/// The end table & the `tail` must be within the `bytes`, so check their length first.
///
/// # Errors
/// Returns an error if the end table decreases, or goes past the end of the `bytes`.
///
/// # Panics
/// Panics if the end table or the `tail` is not within the `bytes`.
pub fn check_ends(
    bytes: &[u8],
    table: usize,
    count: usize,
    tail: usize,
) -> Result<(), DecodeError> {
    let (ends, _): (&[[u8; 4]], &[u8]) = bytes[table..table + count * 4].as_chunks();
    let mut previous: u32 = 0;
    for end in ends {
        let end: u32 = u32::from_le_bytes(*end);
        if end < previous {
            return Err(DecodeError::InvalidEndTable);
        }
        previous = end;
    }
    if previous as usize > bytes[tail..].len() {
        return Err(DecodeError::InvalidEndTable);
    }
    Ok(())
}

/// Checks the `bytes` are valid UTF-8.
///
/// # Errors
/// Returns an error if the `bytes` are not valid UTF-8.
pub const fn check_utf8(bytes: &[u8]) -> Result<(), DecodeError> {
    match std::str::from_utf8(bytes) {
        Ok(_) => Ok(()),
        Err(_) => Err(DecodeError::InvalidUtf8),
    }
}

/// Checks the `value` of an enum is a valid tag number.
///
/// # Errors
/// Returns an error if the `value` is not a valid tag number.
pub const fn check_tag_number(value: u16) -> Result<(), DecodeError> {
    if TagNumber::is_valid(value) {
        Ok(())
    } else {
        Err(DecodeError::InvalidTagNumber { value })
    }
}

/// Checks the `value` of an optional enum is a valid tag number, or 0 for an absent value.
///
/// # Errors
/// Returns an error if the `value` is not 0 or a valid tag number.
pub const fn check_optional_tag_number(value: u16) -> Result<(), DecodeError> {
    if value == 0 {
        Ok(())
    } else {
        check_tag_number(value)
    }
}
