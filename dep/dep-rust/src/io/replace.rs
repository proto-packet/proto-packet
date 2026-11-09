use crate::io::CapacityError;

/// Reads the end offset at the `offset`.
pub(crate) fn read_end(bytes: &[u8], offset: usize) -> usize {
    let end: &[u8; 4] = bytes[offset..].first_chunk().expect("end out of bounds");
    u32::from_le_bytes(*end) as usize
}

/// Replaces the string `entry` with the `value`, in place.
///
/// # Errors
/// Returns an error, leaving the `entry` unchanged, if the `value` isn't the entry's length.
pub fn replace_str(entry: &mut [u8], value: &str) -> Result<(), CapacityError> {
    if value.len() != entry.len() {
        return Err(CapacityError {
            required: value.len(),
            capacity: entry.len(),
        });
    }
    entry.copy_from_slice(value.as_bytes());
    Ok(())
}

/// Replaces the case of the variant `bytes` with the `tag` & the `value`, in place.
///
/// # Errors
/// Returns an error, leaving the `bytes` unchanged, if the variant would change its length.
pub fn replace_variant(bytes: &mut [u8], tag: u16, value: &[u8]) -> Result<(), CapacityError> {
    let required: usize = 2 + value.len();
    let capacity: usize = bytes.len();
    if required != capacity {
        return Err(CapacityError { required, capacity });
    }
    bytes[..2].copy_from_slice(&tag.to_le_bytes());
    bytes[2..].copy_from_slice(value);
    Ok(())
}
