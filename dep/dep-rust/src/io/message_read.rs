use crate::io::{
    DecodeError, TagNumber, check_ends, check_exact_len, check_len, read_u16_unchecked,
    read_u32_unchecked,
};
use std::ops::Range;

/// Gets the offset of the ends of a message with the `count` of fields.
const fn ends_offset(count: usize) -> usize {
    2 + 2 * count
}

/// Gets the offset of the values of a message with the `count` of fields.
///
/// # Note
/// - The last field has no end, since its value ends where the message ends.
const fn values_offset(count: usize) -> usize {
    2 + 2 * count + 4 * count.saturating_sub(1)
}

/// Gets the range of the value at the `index` of the message `bytes` with the `count` of fields.
///
/// # Safety
/// The message index must be valid.
unsafe fn value_range_unchecked(bytes: &[u8], count: usize, index: usize) -> Range<usize> {
    let (ends, values): (usize, usize) = (ends_offset(count), values_offset(count));
    let start: usize = if index == 0 {
        0
    } else {
        unsafe { read_u32_unchecked(bytes, ends + (index - 1) * 4) as usize }
    };
    let end: usize = if index + 1 == count {
        bytes.len() - values
    } else {
        unsafe { read_u32_unchecked(bytes, ends + index * 4) as usize }
    };
    values + start..values + end
}

/// Gets the length of the index of a message with the `count` of fields.
#[inline]
#[must_use]
pub const fn message_index_len(count: usize) -> usize {
    values_offset(count)
}

/// Checks the index of the message `bytes`.
///
/// # Errors
/// Returns an error if the `bytes` are shorter than the index, the tags aren't strictly increasing
/// valid tag numbers, the end table is invalid, or a message without fields has more bytes.
#[inline]
pub fn check_message(bytes: &[u8]) -> Result<(), DecodeError> {
    check_len(bytes, 2)?;
    let count: usize = unsafe { read_u16_unchecked(bytes, 0) as usize };
    if count == 0 {
        return check_exact_len(bytes, 2);
    }
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
    check_ends(
        bytes,
        ends_offset(count),
        count.saturating_sub(1),
        values_offset(count),
    )
}

/// The tags & the values of the fields of a message, in tag order.
#[derive(Clone, Debug)]
pub struct MessageFields<'a> {
    bytes: &'a [u8],
    count: usize,
    index: usize,
}

/// Gets the tags & the values of the fields of the message `bytes`, in tag order.
///
/// # Safety
/// The message index must be valid.
#[inline]
#[must_use]
pub unsafe fn message_fields_unchecked(bytes: &[u8]) -> MessageFields<'_> {
    let count: usize = unsafe { read_u16_unchecked(bytes, 0) as usize };
    MessageFields {
        bytes,
        count,
        index: 0,
    }
}

impl<'a> Iterator for MessageFields<'a> {
    type Item = (u16, &'a [u8]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.count {
            return None;
        }
        let bytes: &'a [u8] = self.bytes;
        let tag: u16 = unsafe { read_u16_unchecked(bytes, 2 + 2 * self.index) };
        let range: Range<usize> = unsafe { value_range_unchecked(bytes, self.count, self.index) };
        let value: &'a [u8] = unsafe { bytes.get_unchecked(range) };
        self.index += 1;
        Some((tag, value))
    }
}

/// Searches the `count` of sorted tags read by `tag_at` for the `tag`.
///
/// Returns the tag's index, or the error of the index to insert it at.
#[inline]
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
    Some(unsafe { value_range_unchecked(bytes, count, index) })
}

/// Finds the value of the field with the `tag` in the message `bytes`.
///
/// # Safety
/// The message index must be valid.
#[inline]
#[must_use]
pub unsafe fn find_field_unchecked(bytes: &[u8], tag: u16) -> Option<&[u8]> {
    let range: Range<usize> = unsafe { find_range_unchecked(bytes, tag) }?;
    Some(unsafe { bytes.get_unchecked(range) })
}

/// Finds the value of the field with the `tag` in the message `bytes` mutably.
///
/// # Safety
/// The message index must be valid.
#[inline]
#[must_use]
pub unsafe fn find_field_mut_unchecked(bytes: &mut [u8], tag: u16) -> Option<&mut [u8]> {
    let range: Range<usize> = unsafe { find_range_unchecked(bytes, tag) }?;
    Some(unsafe { bytes.get_unchecked_mut(range) })
}
