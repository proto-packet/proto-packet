/// Defines the little-endian reads of a fixed-size type.
macro_rules! read {
    ($read:ident, $read_unchecked:ident, $type:ty) => {
        #[doc = concat!("Reads the little-endian `", stringify!($type), "` at the `offset`.")]
        ///
        /// # Panics
        /// Panics if the value is not within the `bytes`.
        pub fn $read(bytes: &[u8], offset: usize) -> $type {
            const SIZE: usize = size_of::<$type>();
            let value: &[u8; SIZE] = bytes[offset..].first_chunk().expect("value out of bounds");
            <$type>::from_le_bytes(*value)
        }

        #[doc = concat!("Reads the little-endian `", stringify!($type), "` at the `offset`.")]
        ///
        /// # Safety
        /// The value must be within the `bytes`.
        pub unsafe fn $read_unchecked(bytes: &[u8], offset: usize) -> $type {
            const SIZE: usize = size_of::<$type>();
            let value: [u8; SIZE] = unsafe { *bytes.as_ptr().add(offset).cast::<[u8; SIZE]>() };
            <$type>::from_le_bytes(value)
        }
    };
}

read!(read_u8, read_u8_unchecked, u8);
read!(read_u16, read_u16_unchecked, u16);
read!(read_u32, read_u32_unchecked, u32);
read!(read_u64, read_u64_unchecked, u64);
read!(read_u128, read_u128_unchecked, u128);
read!(read_i8, read_i8_unchecked, i8);
read!(read_i16, read_i16_unchecked, i16);
read!(read_i32, read_i32_unchecked, i32);
read!(read_i64, read_i64_unchecked, i64);
read!(read_i128, read_i128_unchecked, i128);
read!(read_f32, read_f32_unchecked, f32);
read!(read_f64, read_f64_unchecked, f64);

/// Reads the `bit` of the byte at the `offset`. (bit 0 is the least significant)
///
/// # Panics
/// Panics if the byte is not within the `bytes`, or the `bit` is not less than 8.
pub fn read_bit(bytes: &[u8], offset: usize, bit: u8) -> bool {
    bytes[offset] & (1 << bit) != 0
}

/// Reads the `bit` of the byte at the `offset`. (bit 0 is the least significant)
///
/// # Safety
/// The byte must be within the `bytes`, & the `bit` must be less than 8.
pub unsafe fn read_bit_unchecked(bytes: &[u8], offset: usize, bit: u8) -> bool {
    unsafe { *bytes.get_unchecked(offset) & (1 << bit) != 0 }
}

#[cfg(test)]
mod tests {
    use crate::io::*;

    #[test]
    fn reads_are_little_endian_at_the_offset() {
        let bytes: [u8; 9] = [0xFF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        assert_eq!(read_u8(&bytes, 1), 0x01);
        assert_eq!(read_u16(&bytes, 1), 0x0201);
        assert_eq!(read_u32(&bytes, 1), 0x04030201);
        assert_eq!(read_u64(&bytes, 1), 0x0807060504030201);
        assert_eq!(read_i8(&bytes, 0), -1);
        assert_eq!(read_f32(&bytes, 1), f32::from_bits(0x04030201));
        unsafe {
            assert_eq!(read_u32_unchecked(&bytes, 1), 0x04030201);
            assert_eq!(read_u64_unchecked(&bytes, 1), 0x0807060504030201);
        }

        let bytes: [u8; 16] = (-2_i128).to_le_bytes();
        assert_eq!(read_i128(&bytes, 0), -2);
        assert_eq!(read_u128(&bytes, 0), u128::MAX - 1);
    }

    #[test]
    fn reads_bits_from_the_least_significant() {
        let bytes: [u8; 2] = [0b0000_0000, 0b1000_0001];
        assert!(read_bit(&bytes, 1, 0));
        assert!(!read_bit(&bytes, 1, 1));
        assert!(read_bit(&bytes, 1, 7));
        assert!(!read_bit(&bytes, 0, 0));
        unsafe {
            assert!(read_bit_unchecked(&bytes, 1, 7));
            assert!(!read_bit_unchecked(&bytes, 1, 6));
        }
    }

    #[test]
    #[should_panic(expected = "value out of bounds")]
    fn reads_past_the_end_panic() {
        read_u32(&[0, 1, 2, 3], 1);
    }
}
