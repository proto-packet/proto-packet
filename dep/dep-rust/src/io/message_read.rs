use crate::io::read::entry_range_unchecked;
use crate::io::{DecodeError, TagNumber, UnknownField, check_ends, check_len, read_u16_unchecked};
use std::ops::Range;

/// Gets the offset of the ends of a message with the `count` of fields.
const fn ends_offset(count: usize) -> usize {
    2 + 2 * count
}

/// Gets the offset of the values of a message with the `count` of fields.
const fn values_offset(count: usize) -> usize {
    2 + 6 * count
}

/// Checks the index of the message `bytes`.
///
/// # Errors
/// Returns an error if the `bytes` are shorter than the index, the tags aren't strictly increasing
/// valid tag numbers, or the end table is invalid.
pub fn check_message(bytes: &[u8]) -> Result<(), DecodeError> {
    check_len(bytes, 2)?;
    let count: usize = unsafe { read_u16_unchecked(bytes, 0) as usize };
    check_len(bytes, values_offset(count))?;
    let (tags, _): (&[[u8; 2]], &[u8]) = bytes[2..ends_offset(count)].as_chunks();
    let mut previous: u16 = 0;
    for tag in tags {
        let tag: u16 = u16::from_le_bytes(*tag);
        if tag <= previous || !TagNumber::is_valid(tag) {
            return Err(DecodeError::InvalidTags);
        }
        previous = tag;
    }
    check_ends(bytes, ends_offset(count), count, values_offset(count))
}

/// Searches the `count` of sorted tags read by `tag_at` for the `tag`.
///
/// Returns the tag's index, or the error of the index to insert it at.
pub(crate) fn search_tags<F>(count: usize, tag: u16, tag_at: F) -> Result<usize, usize>
where
    F: Fn(usize) -> u16,
{
    let (mut low, mut high): (usize, usize) = (0, count);
    while low < high {
        let middle: usize = (low + high) / 2;
        let found: u16 = tag_at(middle);
        if found < tag {
            low = middle + 1;
        } else if found > tag {
            high = middle;
        } else {
            return Ok(middle);
        }
    }
    Err(low)
}

/// Finds the range of the value of the field with the `tag`, in the message `bytes`.
///
/// # Safety
/// The message index must be valid.
unsafe fn find_range_unchecked(bytes: &[u8], tag: u16) -> Option<Range<usize>> {
    let count: usize = unsafe { read_u16_unchecked(bytes, 0) as usize };
    let tag_at = |index: usize| unsafe { read_u16_unchecked(bytes, 2 + 2 * index) };
    let index: usize = search_tags(count, tag, tag_at).ok()?;
    let (ends, values): (usize, usize) = (ends_offset(count), values_offset(count));
    Some(unsafe { entry_range_unchecked(bytes, ends, index, values) })
}

/// Finds the value of the field with the `tag` in the message `bytes`.
///
/// # Safety
/// The message index must be valid.
#[must_use]
pub unsafe fn find_field_unchecked(bytes: &[u8], tag: u16) -> Option<&[u8]> {
    let range: Range<usize> = unsafe { find_range_unchecked(bytes, tag) }?;
    Some(unsafe { bytes.get_unchecked(range) })
}

/// Finds the value of the field with the `tag` in the message `bytes` mutably.
///
/// # Safety
/// The message index must be valid.
#[must_use]
pub unsafe fn find_field_mut_unchecked(bytes: &mut [u8], tag: u16) -> Option<&mut [u8]> {
    let range: Range<usize> = unsafe { find_range_unchecked(bytes, tag) }?;
    Some(unsafe { bytes.get_unchecked_mut(range) })
}

/// Reads the fields of the message `bytes` that aren't in the sorted `known` tags.
///
/// # Safety
/// The message index must be valid.
#[must_use]
pub unsafe fn read_unknown_fields_unchecked(bytes: &[u8], known: &[u16]) -> Vec<UnknownField> {
    let count: usize = unsafe { read_u16_unchecked(bytes, 0) as usize };
    let mut result: Vec<UnknownField> = Vec::default();
    let mut next: usize = 0;
    for index in 0..count {
        let tag: u16 = unsafe { read_u16_unchecked(bytes, 2 + 2 * index) };
        while known.get(next).is_some_and(|known| *known < tag) {
            next += 1;
        }
        if known.get(next) != Some(&tag) {
            let (ends, values): (usize, usize) = (ends_offset(count), values_offset(count));
            let range: Range<usize> = unsafe { entry_range_unchecked(bytes, ends, index, values) };
            let value: &[u8] = unsafe { bytes.get_unchecked(range) };
            let tag: TagNumber = unsafe { TagNumber::new_unchecked(tag) };
            result.push(UnknownField::new(tag, value.to_vec()));
        }
    }
    result
}
