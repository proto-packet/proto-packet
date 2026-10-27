use crate::io::DecodeError;
use std::fmt::{Debug, Formatter};

/// A borrowed list of fixed-size elements, read in place. (see `FORMAT.md`)
///
/// The elements are one after another, each read from its bytes by the `read` function. The
/// generated code passes a closure, so the reads are inlined.
#[derive(Copy, Clone)]
pub struct FixedListRef<'a, F> {
    bytes: &'a [u8],
    size: usize,
    read: F,
}

impl<'a, F> FixedListRef<'a, F> {
    //! Construction

    /// Creates the list of the elements of the `size` in the `bytes`, read by the `read` function.
    ///
    /// The list's encoding is validated, but not its elements.
    ///
    /// # Errors
    /// Returns an error if the length of the `bytes` is not a multiple of the `size`.
    ///
    /// # Panics
    /// Panics if the `size` is 0.
    pub fn new(bytes: &'a [u8], size: usize, read: F) -> Result<Self, DecodeError> {
        assert!(size > 0, "element size is 0");
        if bytes.len().is_multiple_of(size) {
            Ok(Self { bytes, size, read })
        } else {
            Err(DecodeError::InvalidList)
        }
    }

    /// Creates the list of the elements of the `size` in the `bytes`, read by the `read` function,
    /// without validating them.
    ///
    /// # Safety
    /// The `size` must not be 0, & the length of the `bytes` must be a multiple of it.
    pub const unsafe fn new_unchecked(bytes: &'a [u8], size: usize, read: F) -> Self {
        Self { bytes, size, read }
    }
}

impl<'a, T, F> FixedListRef<'a, F>
where
    F: Fn(&'a [u8]) -> T,
{
    //! Elements

    /// Gets the number of elements.
    pub fn len(&self) -> usize {
        self.bytes.len() / self.size
    }

    /// Checks if there are no elements.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Gets the optional element at the `index`.
    pub fn get(&self, index: usize) -> Option<T> {
        (index < self.len()).then(|| unsafe { self.get_unchecked(index) })
    }

    /// Gets the element at the `index`, without checking the bounds.
    ///
    /// # Safety
    /// The `index` must be less than the number of elements.
    pub unsafe fn get_unchecked(&self, index: usize) -> T {
        let start: usize = index * self.size;
        (self.read)(unsafe { self.bytes.get_unchecked(start..start + self.size) })
    }

    /// Iterates the elements.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = T> + DoubleEndedIterator + '_ {
        self.bytes.chunks_exact(self.size).map(&self.read)
    }

    /// Copies the elements into a `Vec`.
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().collect()
    }
}

impl<'a, T, F> Debug for FixedListRef<'a, F>
where
    F: Fn(&'a [u8]) -> T,
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{DecodeError, FixedListRef, read_u32};

    /// Reads a `u32` element.
    fn element(bytes: &[u8]) -> u32 {
        read_u32(bytes, 0)
    }

    #[test]
    fn elements_are_read_in_order() {
        let bytes: Vec<u8> = [1_u32, 2, 3].iter().flat_map(|n| n.to_le_bytes()).collect();
        let list = FixedListRef::new(&bytes, 4, element).unwrap();
        assert_eq!(list.len(), 3);
        assert!(!list.is_empty());
        assert_eq!(list.get(1), Some(2));
        assert_eq!(list.get(3), None);
        assert_eq!(list.to_vec(), [1, 2, 3]);
        assert_eq!(list.iter().rev().collect::<Vec<u32>>(), [3, 2, 1]);
        assert_eq!(format!("{:?}", list), "[1, 2, 3]");

        let list = FixedListRef::new(&[], 4, element).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn lengths_must_be_multiples_of_the_size() {
        assert!(matches!(
            FixedListRef::new(&[0; 5], 4, element),
            Err(DecodeError::InvalidList)
        ));
    }
}
