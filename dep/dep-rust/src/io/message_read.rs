use crate::io::{
    DecodeError, TagNumber, UnknownField, check_ends, check_len, read_u16_unchecked,
    read_u32_unchecked,
};

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

/// Finds the index of the `tag` in the message `bytes` with the `count` of fields.
///
/// # Safety
/// The message index must be valid.
unsafe fn find_index_unchecked(bytes: &[u8], count: usize, tag: u16) -> Option<usize> {
    let (mut low, mut high): (usize, usize) = (0, count);
    while low < high {
        let middle: usize = (low + high) / 2;
        let found: u16 = unsafe { read_u16_unchecked(bytes, 2 + 2 * middle) };
        if found < tag {
            low = middle + 1;
        } else if found > tag {
            high = middle;
        } else {
            return Some(middle);
        }
    }
    None
}

/// Gets the range of the value at the `index` of the message `bytes` with the `count` of fields.
///
/// # Safety
/// The message index must be valid, & the `index` must be less than the `count`.
unsafe fn value_range_unchecked(
    bytes: &[u8],
    count: usize,
    index: usize,
) -> std::ops::Range<usize> {
    let ends: usize = ends_offset(count);
    let start: usize = if index == 0 {
        0
    } else {
        unsafe { read_u32_unchecked(bytes, ends + (index - 1) * 4) as usize }
    };
    let end: usize = unsafe { read_u32_unchecked(bytes, ends + index * 4) as usize };
    values_offset(count) + start..values_offset(count) + end
}

/// Finds the value of the field with the `tag` in the message `bytes`.
///
/// # Safety
/// The message index must be valid.
#[must_use]
pub unsafe fn find_field_unchecked(bytes: &[u8], tag: u16) -> Option<&[u8]> {
    let count: usize = unsafe { read_u16_unchecked(bytes, 0) as usize };
    let index: usize = unsafe { find_index_unchecked(bytes, count, tag) }?;
    let range: std::ops::Range<usize> = unsafe { value_range_unchecked(bytes, count, index) };
    Some(unsafe { bytes.get_unchecked(range) })
}

/// Finds the value of the field with the `tag` in the message `bytes` mutably.
///
/// # Safety
/// The message index must be valid.
#[must_use]
pub unsafe fn find_field_mut_unchecked(bytes: &mut [u8], tag: u16) -> Option<&mut [u8]> {
    let count: usize = unsafe { read_u16_unchecked(bytes, 0) as usize };
    let index: usize = unsafe { find_index_unchecked(bytes, count, tag) }?;
    let range: std::ops::Range<usize> = unsafe { value_range_unchecked(bytes, count, index) };
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
            let range: std::ops::Range<usize> =
                unsafe { value_range_unchecked(bytes, count, index) };
            let value: &[u8] = unsafe { bytes.get_unchecked(range) };
            let tag: TagNumber = unsafe { TagNumber::new_unchecked(tag) };
            result.push(UnknownField::new(tag, value.to_vec()));
        }
    }
    result
}
