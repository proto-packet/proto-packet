use crate::io::message_read::search_tags;
use crate::io::replace::read_end;
use crate::io::{CapacityError, message_index_len};

/// Reads the `u16` at the `offset`.
fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    let value: &[u8; 2] = bytes[offset..].first_chunk().expect("u16 out of bounds");
    u16::from_le_bytes(*value)
}

/// Replaces the value of the field with the `tag` in the message `bytes`, in place.
///
/// # Note
/// - Nothing moves in place, so the message must stay the same length.
/// - A `None` `value` is an absent field.
///
/// # Errors
/// Returns an error, leaving the `bytes` unchanged, if the message would change its length. The
/// error has the length it would need.
///
/// # Panics
/// Panics if the message index is not valid & within the `bytes`.
pub fn replace_field(
    bytes: &mut [u8],
    tag: u16,
    value: Option<&[u8]>,
) -> Result<(), CapacityError> {
    let count: usize = read_u16(bytes, 0) as usize;
    let tag_at = |index: usize| read_u16(bytes, 2 + 2 * index);
    let index: Option<usize> = search_tags(count, tag, tag_at).ok();
    let capacity: usize = bytes.len();
    let Some(index) = index else {
        return match value {
            Some(value) => Err(CapacityError {
                required: capacity - message_index_len(count)
                    + message_index_len(count + 1)
                    + value.len(),
                capacity,
            }),
            None => Ok(()),
        };
    };

    let ends: usize = 2 + 2 * count;
    let values: usize = message_index_len(count);
    let start: usize = match index {
        0 => 0,
        _ => read_end(bytes, ends + (index - 1) * 4),
    };
    let end: usize = if index + 1 == count {
        capacity - values
    } else {
        read_end(bytes, ends + index * 4)
    };
    match value {
        Some(value) if value.len() == end - start => {
            bytes[values + start..values + end].copy_from_slice(value);
            Ok(())
        }
        Some(value) => Err(CapacityError {
            required: capacity - (end - start) + value.len(),
            capacity,
        }),
        None => Err(CapacityError {
            required: capacity - (end - start) - message_index_len(count)
                + message_index_len(count - 1),
            capacity,
        }),
    }
}
