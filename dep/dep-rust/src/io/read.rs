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

/// Reads the tail entry at the `index`, of the end table at the `table`, for the tail at the `tail`.
///
/// # Safety
/// The end table must be valid & within the `bytes`, & the `tail` must be within the `bytes`.
#[must_use]
pub unsafe fn read_entry_unchecked(bytes: &[u8], table: usize, index: usize, tail: usize) -> &[u8] {
    let start: usize = if index == 0 {
        0
    } else {
        unsafe { read_u32_unchecked(bytes, table + (index - 1) * 4) as usize }
    };
    let end: usize = unsafe { read_u32_unchecked(bytes, table + index * 4) as usize };
    unsafe { bytes.get_unchecked(tail + start..tail + end) }
}

/// Reads the string entry at the `index`, of the end table at the `table`, for the tail at the
/// `tail`.
///
/// # Safety
/// The end table must be valid & within the `bytes`, the `tail` must be within the `bytes`, & the
/// entry must be valid UTF-8.
#[must_use]
pub unsafe fn read_str_unchecked(bytes: &[u8], table: usize, index: usize, tail: usize) -> &str {
    unsafe { std::str::from_utf8_unchecked(read_entry_unchecked(bytes, table, index, tail)) }
}
