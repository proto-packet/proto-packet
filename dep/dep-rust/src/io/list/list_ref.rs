use crate::io::list::element::stride;
use crate::io::{DecodeError, Element, ListIter, align_up, check_padding};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Range;

/// A read-only view of an encoded list.
///
/// # Note
/// - Fixed-size elements are back to back, so their count comes from the length of the bytes.
/// - Variable-size elements follow their count & an end for each element but the last.
/// - An empty element has no padding before it.
/// - A list has one encoding, so two lists are equal when their bytes are.
pub struct ListRef<'a, T> {
    bytes: &'a [u8],
    element: PhantomData<T>,
}

impl<'a, T> ListRef<'a, T>
where
    T: Element,
{
    //! Construction

    /// Creates a new ref of the `bytes`.
    ///
    /// # Errors
    /// Returns an error if the `bytes` are not a valid encoding.
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        match T::SIZE {
            Some(size) => Self::check_fixed(bytes, size)?,
            None => Self::check_variable(bytes)?,
        }
        Ok(Self {
            bytes,
            element: PhantomData,
        })
    }

    /// Creates a new ref of the `bytes`, without validating them.
    ///
    /// # Safety
    /// The `bytes` must be a valid encoding.
    #[must_use]
    pub const unsafe fn new_unchecked(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            element: PhantomData,
        }
    }

    /// Checks the `bytes` of a list of elements of the fixed `size`.
    fn check_fixed(bytes: &[u8], size: usize) -> Result<(), DecodeError> {
        if bytes.is_empty() {
            return Ok(());
        }
        let stride: usize = stride::<T>(size);
        if size == 0 || bytes.len() < size || !(bytes.len() - size).is_multiple_of(stride) {
            return Err(DecodeError::InvalidList);
        }
        if stride == size && !T::CAN_BE_INVALID {
            return Ok(());
        }
        for start in (0..bytes.len()).step_by(stride) {
            if start > 0 {
                check_padding(&bytes[start - (stride - size)..start])?;
            }
            T::check(&bytes[start..start + size])?;
        }
        Ok(())
    }

    /// Checks the `bytes` of a list of variable-size elements.
    fn check_variable(bytes: &[u8]) -> Result<(), DecodeError> {
        if bytes.is_empty() {
            return Ok(());
        }
        let Some(count) = bytes.first_chunk::<4>() else {
            return Err(DecodeError::TooShort {
                expected: 4,
                actual: bytes.len(),
            });
        };
        let count: usize = u32::from_le_bytes(*count) as usize;
        let Some(table) = count.checked_mul(4).filter(|table| *table <= bytes.len()) else {
            return Err(DecodeError::InvalidList);
        };
        if count == 0 {
            return Err(DecodeError::InvalidList);
        }
        let (ends, _): (&[[u8; 4]], &[u8]) = bytes[4..table].as_chunks();
        let mut previous: usize = table;
        for index in 0..count {
            let end: usize = match ends.get(index) {
                Some(end) => u32::from_le_bytes(*end) as usize,
                None => bytes.len(),
            };
            if end < previous || end > bytes.len() {
                return Err(DecodeError::InvalidEndTable);
            }
            let start: usize = if end == previous {
                end
            } else {
                align_up(previous, T::ALIGNMENT)
            };
            if start > end || (start == end && end != previous) {
                return Err(DecodeError::InvalidEndTable);
            }
            check_padding(&bytes[previous..start])?;
            T::check(&bytes[start..end])?;
            previous = end;
        }
        Ok(())
    }
}

impl<T> Copy for ListRef<'_, T> {}

impl<T> Clone for ListRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for ListRef<'_, T> {}

impl<T> PartialEq for ListRef<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<T> Hash for ListRef<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<'a, T> ListRef<'a, T>
where
    T: Element,
{
    //! Properties

    /// Gets the encoded bytes.
    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }

    /// Gets the count of the elements.
    #[must_use]
    pub fn len(self) -> usize {
        if self.bytes.is_empty() {
            return 0;
        }
        match T::SIZE {
            Some(size) => (self.bytes.len() - size) / stride::<T>(size) + 1,
            None => unsafe { crate::io::read_u32_unchecked(self.bytes, 0) as usize },
        }
    }

    /// Checks if the list has no elements.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    /// Gets the element at the `index`.
    ///
    /// Returns `None` if the `index` is not less than the count.
    #[must_use]
    pub fn get(self, index: usize) -> Option<T::Ref<'a>> {
        if index < self.len() {
            Some(unsafe { self.get_unchecked(index) })
        } else {
            None
        }
    }

    /// Gets the element at the `index`, without checking it.
    ///
    /// # Safety
    /// The `index` must be less than the count.
    #[must_use]
    pub unsafe fn get_unchecked(self, index: usize) -> T::Ref<'a> {
        let range: Range<usize> = match T::SIZE {
            Some(size) => {
                let start: usize = index * stride::<T>(size);
                start..start + size
            }
            None => unsafe { self.variable_range_unchecked(index) },
        };
        unsafe { T::read_unchecked(self.bytes.get_unchecked(range)) }
    }

    /// Gets the range of the variable-size element at the `index`.
    ///
    /// # Safety
    /// The `index` must be less than the count.
    unsafe fn variable_range_unchecked(self, index: usize) -> Range<usize> {
        let count: usize = unsafe { crate::io::read_u32_unchecked(self.bytes, 0) as usize };
        let previous: usize = if index == 0 {
            count * 4
        } else {
            unsafe { crate::io::read_u32_unchecked(self.bytes, index * 4) as usize }
        };
        let end: usize = if index + 1 == count {
            self.bytes.len()
        } else {
            unsafe { crate::io::read_u32_unchecked(self.bytes, 4 + index * 4) as usize }
        };
        if end == previous {
            end..end
        } else {
            align_up(previous, T::ALIGNMENT)..end
        }
    }
}

impl<'a, T> ListRef<'a, T>
where
    T: Element,
{
    //! Elements

    /// Gets the iterator of the elements.
    pub fn iter(self) -> ListIter<'a, T> {
        ListIter::new(self)
    }

    /// Decodes the list.
    #[must_use]
    pub fn decode(self) -> Vec<T> {
        self.iter().map(T::decode).collect()
    }
}

impl<'a, T> IntoIterator for ListRef<'a, T>
where
    T: Element,
{
    type Item = T::Ref<'a>;
    type IntoIter = ListIter<'a, T>;

    fn into_iter(self) -> ListIter<'a, T> {
        self.iter()
    }
}

impl<T> Debug for ListRef<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListRef")
            .field("bytes", &self.bytes)
            .finish()
    }
}
