use crate::io::{DecodeError, read_bit_unchecked, read_u32, read_u32_unchecked};
use std::fmt::{Debug, Formatter};

/// The size of a list's count.
const COUNT_SIZE: usize = 4;

/// A borrowed list of `bool` elements, read in place. (see `FORMAT.md`)
///
/// The list is the `u32` count of its elements, followed by a bit for each element.
#[derive(Copy, Clone)]
pub struct BitListRef<'a> {
    len: usize,
    bits: &'a [u8],
}

impl<'a> BitListRef<'a> {
    //! Construction

    /// Creates the list of the `bool` elements encoded in the `bytes`.
    ///
    /// # Errors
    /// Returns an error if the `bytes` are not the size of the count & bits of the elements.
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() < COUNT_SIZE {
            return Err(DecodeError::InvalidList);
        }
        let len: usize = read_u32(bytes, 0) as usize;
        if bytes.len() - COUNT_SIZE == len.div_ceil(8) {
            Ok(Self {
                len,
                bits: &bytes[COUNT_SIZE..],
            })
        } else {
            Err(DecodeError::InvalidList)
        }
    }

    /// Creates the list of the `bool` elements encoded in the `bytes`, without validating them.
    ///
    /// # Safety
    /// The `bytes` must be the size of the count & bits of the elements.
    pub unsafe fn new_unchecked(bytes: &'a [u8]) -> Self {
        Self {
            len: unsafe { read_u32_unchecked(bytes, 0) } as usize,
            bits: unsafe { bytes.get_unchecked(COUNT_SIZE..) },
        }
    }
}

impl BitListRef<'_> {
    //! Elements

    /// Gets the number of elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks if there are no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Gets the optional element at the `index`.
    pub fn get(&self, index: usize) -> Option<bool> {
        (index < self.len).then(|| unsafe { self.get_unchecked(index) })
    }

    /// Gets the element at the `index`, without checking the bounds.
    ///
    /// # Safety
    /// The `index` must be less than the number of elements.
    pub unsafe fn get_unchecked(&self, index: usize) -> bool {
        unsafe { read_bit_unchecked(self.bits, index / 8, (index % 8) as u8) }
    }

    /// Iterates the elements.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = bool> + DoubleEndedIterator + '_ {
        (0..self.len).map(|index| unsafe { self.get_unchecked(index) })
    }

    /// Copies the elements into a `Vec`.
    pub fn to_vec(&self) -> Vec<bool> {
        self.iter().collect()
    }
}

impl Debug for BitListRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{BitListRef, DecodeError};

    #[test]
    fn elements_are_bits_from_the_least_significant() {
        let bytes: [u8; 6] = [9, 0, 0, 0, 0b0000_0101, 0b1111_1111];
        let list: BitListRef = BitListRef::new(&bytes).unwrap();
        assert_eq!(list.len(), 9);
        assert_eq!(list.get(0), Some(true));
        assert_eq!(list.get(1), Some(false));
        assert_eq!(list.get(8), Some(true));
        assert_eq!(list.get(9), None);
        assert_eq!(
            list.to_vec(),
            [true, false, true, false, false, false, false, false, true]
        );
        assert!(BitListRef::new(&[0; 4]).unwrap().is_empty());
    }

    #[test]
    fn the_bits_must_match_the_count() {
        for bytes in [&[0_u8; 3][..], &[9, 0, 0, 0, 0], &[1, 0, 0, 0, 0, 0]] {
            assert!(matches!(
                BitListRef::new(bytes),
                Err(DecodeError::InvalidList)
            ));
        }
    }
}
