/// Rounds the `value` up to a multiple of the `alignment`, a power of 2.
#[must_use]
pub const fn align_up(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

/// Appends the `count` of zeros to the `out` bytes.
pub fn write_zeros(out: &mut Vec<u8>, count: usize) {
    out.resize(out.len() + count, 0);
}

/// Pads the `out` bytes with zeros, so the bytes after the `start` are a multiple of the
/// `alignment`.
pub fn write_padding(out: &mut Vec<u8>, start: usize, alignment: usize) {
    out.resize(start + align_up(out.len() - start, alignment), 0);
}

/// Removes the padding written before an entry, if the entry then wrote nothing.
///
/// The `out` bytes had the `before` length before the padding, & the `padded` length after it.
///
/// # Note
/// - An empty entry has no padding before it.
pub fn trim_padding(out: &mut Vec<u8>, before: usize, padded: usize) {
    if out.len() == padded {
        out.truncate(before);
    }
}

/// Writes the end offset of the tail entry that ends at the end of the `out` bytes, for the tail at
/// the `tail`, to the end table item at the `offset`.
///
/// # Panics
/// Panics if the end offset does not fit in a `u32`.
pub fn write_end(out: &mut [u8], offset: usize, tail: usize) {
    let end: u32 = u32::try_from(out.len() - tail).expect("the tail is too long");
    out[offset..offset + 4].copy_from_slice(&end.to_le_bytes());
}

/// Defines the little-endian write of an integer type.
macro_rules! write_int {
    ($write:ident, $type:ty) => {
        #[doc = concat!("Writes the little-endian `", stringify!($type), "` at the `offset`.")]
        ///
        /// # Safety
        /// The value must be within the `bytes`.
        pub unsafe fn $write(bytes: &mut [u8], offset: usize, value: $type) {
            unsafe { write_bytes_unchecked(bytes, offset, value.to_le_bytes()) }
        }
    };
}

write_int!(write_u8_unchecked, u8);
write_int!(write_u16_unchecked, u16);
write_int!(write_u32_unchecked, u32);
write_int!(write_u64_unchecked, u64);
write_int!(write_u128_unchecked, u128);
write_int!(write_i8_unchecked, i8);
write_int!(write_i16_unchecked, i16);
write_int!(write_i32_unchecked, i32);
write_int!(write_i64_unchecked, i64);
write_int!(write_i128_unchecked, i128);

/// Writes the `N` bytes of the `value` at the `offset`.
///
/// # Safety
/// The bytes must be within the `bytes`.
pub unsafe fn write_bytes_unchecked<const N: usize>(
    bytes: &mut [u8],
    offset: usize,
    value: [u8; N],
) {
    unsafe { *bytes.as_mut_ptr().add(offset).cast::<[u8; N]>() = value }
}

/// Writes the `value` to the `bit` of the byte at the `offset`. (bit 0 is the least significant)
///
/// # Safety
/// The byte must be within the `bytes`, & the `bit` must be less than 8.
pub unsafe fn write_bit_unchecked(bytes: &mut [u8], offset: usize, bit: u8, value: bool) {
    let byte: &mut u8 = unsafe { bytes.get_unchecked_mut(offset) };
    *byte = (*byte & !(1 << bit)) | (u8::from(value) << bit);
}

/// Defines the little-endian write of a float or time type of the runtime.
#[cfg(feature = "types")]
macro_rules! write_float {
    ($write:ident, $float:ident, $name:literal) => {
        #[doc = concat!("Writes the little-endian `", $name, "` at the `offset`.")]
        ///
        /// # Safety
        /// The value must be within the `bytes`.
        pub unsafe fn $write(bytes: &mut [u8], offset: usize, value: crate::types::$float) {
            unsafe { write_bytes_unchecked(bytes, offset, value.to_le_bytes()) }
        }
    };
}

#[cfg(feature = "types")]
write_float!(write_f32_unchecked, Float32, "f32");
#[cfg(feature = "types")]
write_float!(write_f64_unchecked, Float64, "f64");

#[cfg(feature = "types")]
write_float!(write_timestamp_unchecked, Timestamp, "timestamp");
#[cfg(feature = "types")]
write_float!(write_microtime_unchecked, Microtime, "microtime");
#[cfg(feature = "types")]
write_float!(write_nanotime_unchecked, Nanotime, "nanotime");
#[cfg(feature = "types")]
write_float!(write_date_unchecked, Date, "date");
#[cfg(feature = "types")]
write_float!(write_duration_unchecked, Duration, "duration");

/// Writes the little-endian `decimal` at the `offset`.
///
/// # Safety
/// The value must be within the `bytes`.
#[cfg(feature = "types")]
pub unsafe fn write_decimal_unchecked<const S: u8>(
    bytes: &mut [u8],
    offset: usize,
    value: crate::types::Decimal<S>,
) {
    unsafe { write_bytes_unchecked(bytes, offset, value.to_le_bytes()) }
}
