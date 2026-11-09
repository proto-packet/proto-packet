use crate::io::align_up;
use std::ops::Range;

/// Defines the little-endian read of an integer type.
macro_rules! read {
    ($read:ident, $type:ty) => {
        #[doc = concat!("Reads the little-endian `", stringify!($type), "` at the `offset`.")]
        ///
        /// # Safety
        /// The value must be within the `bytes`.
        #[must_use]
        pub unsafe fn $read(bytes: &[u8], offset: usize) -> $type {
            <$type>::from_le_bytes(unsafe { read_bytes_unchecked(bytes, offset) })
        }
    };
}

read!(read_u8_unchecked, u8);
read!(read_u16_unchecked, u16);
read!(read_u32_unchecked, u32);
read!(read_u64_unchecked, u64);
read!(read_u128_unchecked, u128);

/// Reads the `N` bytes at the `offset`.
///
/// # Safety
/// The bytes must be within the `bytes`.
#[must_use]
pub unsafe fn read_bytes_unchecked<const N: usize>(bytes: &[u8], offset: usize) -> [u8; N] {
    unsafe { *bytes.as_ptr().add(offset).cast::<[u8; N]>() }
}

/// Reads the `bit` of the byte at the `offset`. (bit 0 is the least significant)
///
/// # Safety
/// The byte must be within the `bytes`, & the `bit` must be less than 8.
#[must_use]
pub unsafe fn read_bit_unchecked(bytes: &[u8], offset: usize, bit: u8) -> bool {
    unsafe { *bytes.get_unchecked(offset) & (1 << bit) != 0 }
}

/// Gets the range of the entry at the `index` of a struct's tail.
///
/// # Note
/// - The end table at the `table` has the end of each of the `count` of entries but the last.
/// - The entry starts at the next multiple of its `alignment` after the entry before it, or after
///   the `header`.
/// - The last entry ends where the `bytes` end.
///
/// # Safety
/// The end table must be within the `bytes`.
#[inline]
pub(crate) unsafe fn tail_entry_range_unchecked(
    bytes: &[u8],
    table: usize,
    index: usize,
    count: usize,
    header: usize,
    alignment: usize,
) -> Range<usize> {
    let previous: usize = if index == 0 {
        header
    } else {
        unsafe { read_u32_unchecked(bytes, table + (index - 1) * 4) as usize }
    };
    let end: usize = if index + 1 == count {
        bytes.len()
    } else {
        unsafe { read_u32_unchecked(bytes, table + index * 4) as usize }
    };
    align_up(previous, alignment)..end
}

/// Reads the entry at the `index` of a struct's tail.
///
/// # Safety
/// The end table must be valid & within the `bytes`, & the entry must not start after it ends.
#[inline]
#[must_use]
pub unsafe fn read_tail_entry_unchecked(
    bytes: &[u8],
    table: usize,
    index: usize,
    count: usize,
    header: usize,
    alignment: usize,
) -> &[u8] {
    let range: Range<usize> =
        unsafe { tail_entry_range_unchecked(bytes, table, index, count, header, alignment) };
    unsafe { bytes.get_unchecked(range) }
}

/// Reads the entry at the `index` of a struct's tail mutably.
///
/// # Safety
/// The end table must be valid & within the `bytes`, & the entry must not start after it ends.
#[inline]
#[must_use]
pub unsafe fn read_tail_entry_mut_unchecked(
    bytes: &mut [u8],
    table: usize,
    index: usize,
    count: usize,
    header: usize,
    alignment: usize,
) -> &mut [u8] {
    let range: Range<usize> =
        unsafe { tail_entry_range_unchecked(bytes, table, index, count, header, alignment) };
    unsafe { bytes.get_unchecked_mut(range) }
}

/// Reads the string entry at the `index` of a struct's tail.
///
/// # Safety
/// The end table must be valid & within the `bytes`, & the entry must be valid UTF-8.
#[inline]
#[must_use]
pub unsafe fn read_tail_str_unchecked(
    bytes: &[u8],
    table: usize,
    index: usize,
    count: usize,
    header: usize,
) -> &str {
    let entry: &[u8] = unsafe { read_tail_entry_unchecked(bytes, table, index, count, header, 1) };
    unsafe { std::str::from_utf8_unchecked(entry) }
}
