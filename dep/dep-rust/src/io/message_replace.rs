use crate::io::CapacityError;

/// Reads the `u16` at the `offset`.
fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    let value: &[u8; 2] = bytes[offset..].first_chunk().expect("u16 out of bounds");
    u16::from_le_bytes(*value)
}

/// Reads the `u32` at the `offset`.
fn read_u32(bytes: &[u8], offset: usize) -> usize {
    let value: &[u8; 4] = bytes[offset..].first_chunk().expect("u32 out of bounds");
    u32::from_le_bytes(*value) as usize
}

/// Writes the `u32` to the `offset`.
fn write_u32(bytes: &mut [u8], offset: usize, value: usize) {
    bytes[offset..offset + 4].copy_from_slice(&(value as u32).to_le_bytes());
}

/// Replaces the value of the field with the `tag` in the message `bytes`, in place.
///
/// A `None` `value` removes the field, & a new field is added at its place in the tags. Values that
/// keep their length don't move anything. Otherwise the index & the other values move, the message
/// can grow to the end of the `bytes`, & freed bytes are zeroed.
///
/// # Errors
/// Returns an error, leaving the `bytes` unchanged, if the message would not fit.
///
/// # Panics
/// Panics if the message index is not valid & within the `bytes`.
pub fn replace_field(
    bytes: &mut [u8],
    tag: u16,
    value: Option<&[u8]>,
) -> Result<(), CapacityError> {
    let count: usize = read_u16(bytes, 0) as usize;
    let mut index: usize = 0;
    while index < count && read_u16(bytes, 2 + 2 * index) < tag {
        index += 1;
    }
    let is_present: bool = index < count && read_u16(bytes, 2 + 2 * index) == tag;
    if !is_present && value.is_none() {
        return Ok(());
    }

    let old_ends: usize = 2 + 2 * count;
    let old_values: usize = 2 + 6 * count;
    let start: usize = match index {
        0 => 0,
        _ => read_u32(bytes, old_ends + (index - 1) * 4),
    };
    let end: usize = if is_present {
        read_u32(bytes, old_ends + index * 4)
    } else {
        start
    };
    let last: usize = match count {
        0 => 0,
        _ => read_u32(bytes, old_ends + (count - 1) * 4),
    };

    let new_count: usize = match (is_present, value.is_some()) {
        (true, false) => count - 1,
        (false, true) => count + 1,
        _ => count,
    };
    let new_ends: usize = 2 + 2 * new_count;
    let new_values: usize = 2 + 6 * new_count;
    let new_len: usize = value.map_or(0, <[u8]>::len);
    let new_end: usize = start + new_len;
    let new_last: usize = last - (end - start) + new_len;
    if new_count == count && new_len == end - start {
        if let Some(value) = value {
            bytes[old_values + start..old_values + end].copy_from_slice(value);
        }
        return Ok(());
    }

    let required: usize = new_values + new_last;
    let capacity: usize = bytes.len().min(new_values + u32::MAX as usize);
    if required > capacity {
        return Err(CapacityError { required, capacity });
    }

    let old_used: usize = old_values + last;
    if new_count > count {
        // Grow: the values move first, leaving room for the index.
        bytes.copy_within(old_values + end..old_used, new_values + new_end);
        bytes.copy_within(old_values..old_values + start, new_values);
        bytes.copy_within(
            old_ends + index * 4..old_ends + count * 4,
            new_ends + (index + 1) * 4,
        );
        bytes.copy_within(old_ends..old_ends + index * 4, new_ends);
        bytes.copy_within(2 + 2 * index..old_ends, 2 + 2 * (index + 1));
    } else {
        // Shrink: the index moves first, since the values are about to move over it.
        if new_count < count {
            bytes.copy_within(2 + 2 * (index + 1)..old_ends, 2 + 2 * index);
            bytes.copy_within(old_ends..old_ends + index * 4, new_ends);
            bytes.copy_within(
                old_ends + (index + 1) * 4..old_ends + count * 4,
                new_ends + index * 4,
            );
        }
        if new_values + new_end > old_values + end {
            bytes.copy_within(old_values + end..old_used, new_values + new_end);
            bytes.copy_within(old_values..old_values + start, new_values);
        } else {
            bytes.copy_within(old_values..old_values + start, new_values);
            bytes.copy_within(old_values + end..old_used, new_values + new_end);
        }
    }

    if let Some(value) = value {
        bytes[new_values + start..new_values + new_end].copy_from_slice(value);
        bytes[2 + 2 * index..2 + 2 * index + 2].copy_from_slice(&tag.to_le_bytes());
    }
    bytes[0..2].copy_from_slice(&(new_count as u16).to_le_bytes());

    let first_moved: usize = if value.is_some() { index + 1 } else { index };
    if value.is_some() {
        write_u32(bytes, new_ends + index * 4, new_end);
    }
    for item in first_moved..new_count {
        let moved: usize = read_u32(bytes, new_ends + item * 4) - end + new_end;
        write_u32(bytes, new_ends + item * 4, moved);
    }
    if required < old_used {
        bytes[required..old_used].fill(0);
    }
    Ok(())
}
