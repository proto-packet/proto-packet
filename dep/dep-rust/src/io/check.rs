use crate::io::read::tail_entry_range_unchecked;
use crate::io::{DecodeError, TagNumber};
use std::ops::Range;

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

/// Checks the `bytes` are exactly the `size` of a fixed-size value.
///
/// # Errors
/// Returns an error if the `bytes` are shorter or longer than the `size`.
pub const fn check_exact_len(bytes: &[u8], size: usize) -> Result<(), DecodeError> {
    if bytes.len() > size {
        return Err(DecodeError::TooLong {
            expected: size,
            actual: bytes.len(),
        });
    }
    check_len(bytes, size)
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

/// Checks the end table of a struct's tail, at the `table` with the `count` of ends.
///
/// The end table must be within the `bytes`, so check their length first.
///
/// # Errors
/// Returns an error if the end table starts before the end of the `header`, decreases, or goes past
/// the end of the `bytes`.
///
/// # Panics
/// Panics if the end table is not within the `bytes`.
pub fn check_tail_ends(
    bytes: &[u8],
    table: usize,
    count: usize,
    header: usize,
) -> Result<(), DecodeError> {
    let (ends, _): (&[[u8; 4]], &[u8]) = bytes[table..table + count * 4].as_chunks();
    let mut previous: usize = header;
    for end in ends {
        let end: usize = u32::from_le_bytes(*end) as usize;
        if end < previous {
            return Err(DecodeError::InvalidEndTable);
        }
        previous = end;
    }
    if previous > bytes.len() {
        return Err(DecodeError::InvalidEndTable);
    }
    Ok(())
}

/// Checks the entry at the `index` of a struct's tail, & gets it.
///
/// The end table at the `table` must be valid, so check it first. The `count` is of the entries.
///
/// # Errors
/// Returns an error if the entry starts after it ends, once its padding is counted.
pub fn check_tail_entry(
    bytes: &[u8],
    table: usize,
    index: usize,
    count: usize,
    header: usize,
    alignment: usize,
) -> Result<&[u8], DecodeError> {
    let range: Range<usize> =
        unsafe { tail_entry_range_unchecked(bytes, table, index, count, header, alignment) };
    if range.start > range.end {
        return Err(DecodeError::InvalidEndTable);
    }
    let previous: usize = tail_entry_previous(bytes, table, index, header);
    check_padding(&bytes[previous..range.start])?;
    Ok(&bytes[range])
}

/// Gets where the entry before the entry at the `index` of a struct's tail ends.
fn tail_entry_previous(bytes: &[u8], table: usize, index: usize, header: usize) -> usize {
    match index {
        0 => header,
        _ => {
            let end: &[u8; 4] = bytes[table + (index - 1) * 4..]
                .first_chunk()
                .expect("end out of bounds");
            u32::from_le_bytes(*end) as usize
        }
    }
}

/// Checks the absent entry at the `index` of a struct's tail is empty.
///
/// The end table at the `table` must be valid, so check it first. The `count` is of the entries.
///
/// # Errors
/// Returns an error if the entry is not empty.
pub fn check_tail_absent(
    bytes: &[u8],
    table: usize,
    index: usize,
    count: usize,
    header: usize,
) -> Result<(), DecodeError> {
    let range: Range<usize> =
        unsafe { tail_entry_range_unchecked(bytes, table, index, count, header, 1) };
    if range.start == range.end {
        Ok(())
    } else {
        Err(DecodeError::InvalidAbsentField)
    }
}

/// Checks the `padding` is zeros.
///
/// # Errors
/// Returns an error if a byte is not 0.
pub fn check_padding(padding: &[u8]) -> Result<(), DecodeError> {
    if padding.iter().all(|byte| *byte == 0) {
        Ok(())
    } else {
        Err(DecodeError::InvalidPadding)
    }
}

/// Checks the bits of the `byte` from the `used` count of bits up are 0.
///
/// # Errors
/// Returns an error if an unused bit is not 0.
pub const fn check_unused_bits(byte: u8, used: u8) -> Result<(), DecodeError> {
    if byte >> used == 0 {
        Ok(())
    } else {
        Err(DecodeError::InvalidPadding)
    }
}

/// Checks the bytes of an `absent` fixed-size field are zeros.
///
/// # Errors
/// Returns an error if a byte is not 0.
pub fn check_absent(absent: &[u8]) -> Result<(), DecodeError> {
    if absent.iter().all(|byte| *byte == 0) {
        Ok(())
    } else {
        Err(DecodeError::InvalidAbsentField)
    }
}

/// Checks the `byte` of a `bool` is 0 or 1.
///
/// # Errors
/// Returns an error if the `byte` is above 1.
pub const fn check_bool(byte: u8) -> Result<(), DecodeError> {
    if byte <= 1 {
        Ok(())
    } else {
        Err(DecodeError::InvalidBool { value: byte })
    }
}

/// Checks the `bits` of an `f32` are not a NaN that isn't the one NaN.
///
/// # Errors
/// Returns an error if the `bits` are another NaN.
#[cfg(feature = "types")]
pub const fn check_f32(bits: u32) -> Result<(), DecodeError> {
    if crate::types::Float32::is_valid_bits(bits) {
        Ok(())
    } else {
        Err(DecodeError::InvalidFloat)
    }
}

/// Checks the `bits` of an `f64` are not a NaN that isn't the one NaN.
///
/// # Errors
/// Returns an error if the `bits` are another NaN.
#[cfg(feature = "types")]
pub const fn check_f64(bits: u64) -> Result<(), DecodeError> {
    if crate::types::Float64::is_valid_bits(bits) {
        Ok(())
    } else {
        Err(DecodeError::InvalidFloat)
    }
}
