use crate::io::CapacityError;

/// Reads the end offset at the `offset`.
fn read_end(bytes: &[u8], offset: usize) -> usize {
    let end: &[u8; 4] = bytes[offset..].first_chunk().expect("end out of bounds");
    u32::from_le_bytes(*end) as usize
}

/// Replaces the string entry at the `index` with the `value`, in the end table at the `table` with
/// the `count` of entries, for the tail at the `tail`.
///
/// The later entries move, so they must be strings. The tail can grow to the end of the `bytes`, &
/// freed bytes are zeroed.
///
/// # Errors
/// Returns an error, leaving the `bytes` unchanged, if the tail would not fit.
///
/// # Panics
/// Panics if the end table is not valid & within the `bytes`.
pub fn replace_entry(
    bytes: &mut [u8],
    table: usize,
    count: usize,
    index: usize,
    tail: usize,
    value: &str,
) -> Result<(), CapacityError> {
    let start: usize = match index {
        0 => 0,
        _ => read_end(bytes, table + (index - 1) * 4),
    };
    let end: usize = read_end(bytes, table + index * 4);
    let last: usize = read_end(bytes, table + (count - 1) * 4);

    let required: usize = last - (end - start) + value.len();
    let capacity: usize = (bytes.len() - tail).min(u32::MAX as usize);
    if required > capacity {
        return Err(CapacityError { required, capacity });
    }

    let new_end: usize = start + value.len();
    bytes.copy_within(tail + end..tail + last, tail + new_end);
    bytes[tail + start..tail + new_end].copy_from_slice(value.as_bytes());
    for item in index..count {
        let offset: usize = table + item * 4;
        let moved: usize = read_end(bytes, offset) - end + new_end;
        bytes[offset..offset + 4].copy_from_slice(&(moved as u32).to_le_bytes());
    }
    if required < last {
        bytes[tail + required..tail + last].fill(0);
    }
    Ok(())
}

/// Replaces the case of the variant `bytes` with the `tag` & the `value`, in place.
///
/// The rest of the `bytes` are zeroed. An `exact` value must fill the `bytes`, since its length is
/// the rest of the extent. (ex: a string)
///
/// # Errors
/// Returns an error, leaving the `bytes` unchanged, if the case does not fit.
pub fn replace_case(
    bytes: &mut [u8],
    tag: u16,
    value: &[u8],
    exact: bool,
) -> Result<(), CapacityError> {
    let required: usize = 2 + value.len();
    let capacity: usize = bytes.len();
    if required > capacity || (exact && required != capacity) {
        return Err(CapacityError { required, capacity });
    }
    bytes[..2].copy_from_slice(&tag.to_le_bytes());
    bytes[2..required].copy_from_slice(value);
    bytes[required..].fill(0);
    Ok(())
}
