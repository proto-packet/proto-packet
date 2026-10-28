use crate::io::{DecodeError, ListView, read_u32, read_u32_unchecked};
use std::fmt::{Debug, Formatter};

/// The size of a list's count & of each end offset.
const U32_SIZE: usize = 4;

/// A borrowed list of variable-size elements, read in place. (see `FORMAT.md`)
///
/// The list is the `u32` count of its elements, followed by the `u32` end offset of each element,
/// followed by the elements. Each element is read from its bytes by the `read` function. The
/// generated code passes a closure, so the reads are inlined.
#[derive(Copy, Clone)]
pub struct VarListRef<'a, F> {
    len: usize,
    ends: &'a [u8],
    elements: &'a [u8],
    read: F,
}

impl<'a, F> VarListRef<'a, F> {
    //! Construction

    /// Creates the list of the variable-size elements encoded in the `bytes`, read by the `read`
    /// function.
    ///
    /// The list's encoding is validated, but not its elements.
    ///
    /// # Errors
    /// Returns an error if the end offsets do not fit in the `bytes`, decrease, or do not end at
    /// the end of the `bytes`.
    pub fn new(bytes: &'a [u8], read: F) -> Result<Self, DecodeError> {
        if bytes.len() < U32_SIZE {
            return Err(DecodeError::InvalidList);
        }
        let len: usize = read_u32(bytes, 0) as usize;
        let elements_start: usize = len
            .checked_mul(U32_SIZE)
            .and_then(|size| size.checked_add(U32_SIZE))
            .filter(|start| *start <= bytes.len())
            .ok_or(DecodeError::InvalidList)?;
        let ends: &[u8] = &bytes[U32_SIZE..elements_start];
        let elements: &[u8] = &bytes[elements_start..];
        let mut previous: usize = 0;
        for index in 0..len {
            let end: usize = read_u32(ends, index * U32_SIZE) as usize;
            if end < previous {
                return Err(DecodeError::InvalidList);
            }
            previous = end;
        }
        if previous == elements.len() {
            Ok(Self {
                len,
                ends,
                elements,
                read,
            })
        } else {
            Err(DecodeError::InvalidList)
        }
    }

    /// Creates the list of the variable-size elements encoded in the `bytes`, read by the `read`
    /// function, without validating them.
    ///
    /// # Safety
    /// The end offsets must fit in the `bytes`, never decrease, & end at the end of the `bytes`.
    pub unsafe fn new_unchecked(bytes: &'a [u8], read: F) -> Self {
        let len: usize = unsafe { read_u32_unchecked(bytes, 0) } as usize;
        let elements_start: usize = U32_SIZE + len * U32_SIZE;
        Self {
            len,
            ends: unsafe { bytes.get_unchecked(U32_SIZE..elements_start) },
            elements: unsafe { bytes.get_unchecked(elements_start..) },
            read,
        }
    }
}

impl<'a, T, F> VarListRef<'a, F>
where
    F: Fn(&'a [u8]) -> T,
{
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
    pub fn get(&self, index: usize) -> Option<T> {
        (index < self.len).then(|| unsafe { self.get_unchecked(index) })
    }

    /// Gets the element at the `index`, without checking the bounds.
    ///
    /// # Safety
    /// The `index` must be less than the number of elements.
    pub unsafe fn get_unchecked(&self, index: usize) -> T {
        let end =
            |index: usize| unsafe { read_u32_unchecked(self.ends, index * U32_SIZE) } as usize;
        let start: usize = if index == 0 { 0 } else { end(index - 1) };
        (self.read)(unsafe { self.elements.get_unchecked(start..end(index)) })
    }

    /// Iterates the elements.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = T> + DoubleEndedIterator + '_ {
        (0..self.len).map(|index| unsafe { self.get_unchecked(index) })
    }

    /// Copies the elements into a `Vec`.
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().collect()
    }
}

impl<'a, T, F> ListView for VarListRef<'a, F>
where
    F: Fn(&'a [u8]) -> T,
{
    type Item = T;

    fn len(&self) -> usize {
        self.len()
    }

    unsafe fn get_unchecked(&self, index: usize) -> T {
        unsafe { self.get_unchecked(index) }
    }
}

impl<'a, T, F> Debug for VarListRef<'a, F>
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
    use crate::io::{DecodeError, VarListRef};

    /// Encodes the list of the `ends` & `elements`.
    fn encode(ends: &[u32], elements: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = (ends.len() as u32).to_le_bytes().to_vec();
        for end in ends {
            result.extend_from_slice(&end.to_le_bytes());
        }
        result.extend_from_slice(elements);
        result
    }

    /// Reads an element as its bytes.
    fn element(bytes: &[u8]) -> &[u8] {
        bytes
    }

    #[test]
    fn elements_are_between_their_end_offsets() {
        let bytes: Vec<u8> = encode(&[2, 2, 3], b"abc");
        let list = VarListRef::new(&bytes, element).unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(&b"ab"[..]));
        assert_eq!(list.get(1), Some(&b""[..]));
        assert_eq!(list.get(2), Some(&b"c"[..]));
        assert_eq!(list.get(3), None);
        assert_eq!(list.to_vec(), [&b"ab"[..], b"", b"c"]);
        assert!(
            VarListRef::new(&encode(&[], b""), element)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn the_end_offsets_must_be_valid() {
        let test_cases: Vec<Vec<u8>> = vec![
            vec![0; 3],
            encode(&[2, 1], b"ab"),
            encode(&[2, 3], b"ab"),
            encode(&[2], b"abc"),
            [&u32::MAX.to_le_bytes()[..], &[0; 8]].concat(),
        ];
        for bytes in test_cases {
            assert!(
                matches!(
                    VarListRef::new(&bytes, element),
                    Err(DecodeError::InvalidList)
                ),
                "bytes: {:?}",
                bytes
            );
        }
    }
}
