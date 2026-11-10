use crate::io::list::option_write::{
    bits_len, encode_option_list, option_list_alignment, option_list_encoded_len, option_list_start,
};
use crate::io::{ArrayElement, DecodeError, Element, ListRef, MAX_DEPTH};
use crate::io::{check_absent, check_padding, check_unused_bits, read_u32_unchecked};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

/// A read-only view of the encoded optional elements of a list, an array or a map.
///
/// # Note
/// - The presence bits are the last bytes, one for each element, with the low bit first.
/// - An absent element is zeros if it is fixed-size, & no bytes if it isn't.
/// - An element that can't be zeros has no presence bit, since zeros are absent. (ex: an enum)
/// - A list of fixed-size elements with presence bits starts with its count.
pub struct OptionListRef<'a, T> {
    bytes: &'a [u8],
    elements: ListRef<'a, T>,
    bits: &'a [u8],
}

impl<'a, T> OptionListRef<'a, T>
where
    T: Element,
{
    //! Construction

    /// Creates a new ref of the `bytes` of a list.
    ///
    /// # Errors
    /// Returns an error if the `bytes` are not a valid encoding.
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        Self::new_with_depth(bytes, MAX_DEPTH)
    }

    /// Creates a new ref of the `bytes` of a list, whose elements can nest the `depth` of levels.
    ///
    /// # Errors
    /// Returns an error if the `bytes` are not a valid encoding.
    pub fn new_with_depth(bytes: &'a [u8], depth: usize) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Ok(unsafe { Self::of_parts_unchecked(bytes, bytes.len(), bytes.len()) });
        }
        let start: usize = option_list_start::<T>();
        let count: Option<usize> = if T::ZERO_IS_ABSENT {
            None
        } else {
            let Some(count) = bytes.first_chunk::<4>() else {
                return Err(DecodeError::TooShort {
                    expected: 4,
                    actual: bytes.len(),
                });
            };
            Some(u32::from_le_bytes(*count) as usize)
        };
        let bits: usize = count.map_or(0, bits_len::<T>);
        if count == Some(0) || bytes.len() < start + bits {
            return Err(DecodeError::InvalidList);
        }
        if start > 0 {
            check_padding(&bytes[4..start])?;
        }
        let end: usize = bytes.len() - bits;
        Self::check_parts(&bytes[start..end], &bytes[end..], count, depth)?;
        Ok(unsafe { Self::of_parts_unchecked(bytes, start, end) })
    }

    /// Creates a new ref of the `bytes` of a list, without validating them.
    ///
    /// # Safety
    /// The `bytes` must be a valid encoding.
    #[must_use]
    pub unsafe fn new_unchecked(bytes: &'a [u8]) -> Self {
        if bytes.is_empty() || T::ZERO_IS_ABSENT {
            return unsafe { Self::of_parts_unchecked(bytes, 0, bytes.len()) };
        }
        let count: usize = unsafe { read_u32_unchecked(bytes, 0) as usize };
        let end: usize = bytes.len() - bits_len::<T>(count);
        unsafe { Self::of_parts_unchecked(bytes, option_list_start::<T>(), end) }
    }

    /// Creates a new ref of the `bytes` of the `len` of elements, which have no count before them.
    ///
    /// # Note
    /// - An array has its length, & the values of a map have the count of its keys.
    ///
    /// # Errors
    /// Returns an error if the `bytes` are not a valid encoding.
    pub(crate) fn of_len_with_depth(
        bytes: &'a [u8],
        len: usize,
        depth: usize,
    ) -> Result<Self, DecodeError> {
        let bits: usize = bits_len::<T>(len);
        if bytes.len() < bits {
            return Err(DecodeError::InvalidList);
        }
        let end: usize = bytes.len() - bits;
        Self::check_parts(&bytes[..end], &bytes[end..], Some(len), depth)?;
        Ok(unsafe { Self::of_parts_unchecked(bytes, 0, end) })
    }

    /// Creates a new ref of the `bytes` of the `len` of elements, without validating them.
    ///
    /// # Safety
    /// The `bytes` must be a valid encoding of the `len` of elements.
    pub(crate) unsafe fn of_len_unchecked(bytes: &'a [u8], len: usize) -> Self {
        let end: usize = bytes.len() - bits_len::<T>(len);
        unsafe { Self::of_parts_unchecked(bytes, 0, end) }
    }

    /// Creates a new ref of the `bytes`, with the elements from the `start` to the `end` & the
    /// presence bits after them.
    ///
    /// # Safety
    /// The `bytes` must be a valid encoding with the elements & the presence bits at those places.
    unsafe fn of_parts_unchecked(bytes: &'a [u8], start: usize, end: usize) -> Self {
        Self {
            bytes,
            elements: unsafe { ListRef::new_unchecked(bytes.get_unchecked(start..end)) },
            bits: unsafe { bytes.get_unchecked(end..) },
        }
    }

    /// Checks the bytes of the `elements` & their presence `bits`, with the `count` of elements if
    /// something else has it.
    fn check_parts(
        elements: &[u8],
        bits: &[u8],
        count: Option<usize>,
        depth: usize,
    ) -> Result<(), DecodeError> {
        let mut found: usize = 0;
        let check = |index: usize, element: &[u8]| -> Result<(), DecodeError> {
            found = index + 1;
            let is_present: bool = if T::ZERO_IS_ABSENT {
                element.iter().any(|byte| *byte != 0)
            } else {
                bits.get(index / 8)
                    .is_some_and(|byte| byte >> (index % 8) & 1 == 1)
            };
            if is_present {
                T::check(element, depth)
            } else if T::SIZE.is_some() {
                check_absent(element)
            } else if element.is_empty() {
                Ok(())
            } else {
                Err(DecodeError::InvalidAbsentField)
            }
        };
        match T::SIZE {
            Some(size) => ListRef::<T>::check_fixed_with(elements, size, check)?,
            None => ListRef::<T>::check_variable_with(elements, check)?,
        }
        if count.is_some_and(|count| count != found) || bits.len() != bits_len::<T>(found) {
            return Err(DecodeError::InvalidList);
        }
        if let Some(last) = bits.last()
            && !found.is_multiple_of(8)
        {
            check_unused_bits(*last, (found % 8) as u8)?;
        }
        Ok(())
    }
}

impl<T> Copy for OptionListRef<'_, T> {}

impl<T> Clone for OptionListRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for OptionListRef<'_, T> {}

impl<T> PartialEq for OptionListRef<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<T> Hash for OptionListRef<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<'a, T> OptionListRef<'a, T>
where
    T: Element,
{
    //! Properties

    /// Gets the encoded bytes.
    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }

    /// Gets the count of the elements, present or not.
    #[must_use]
    pub fn len(self) -> usize {
        self.elements.len()
    }

    /// Checks if the list has no elements.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.elements.is_empty()
    }

    /// Gets the element at the `index`, which is `None` inside if it is absent.
    ///
    /// Returns `None` if the `index` is not less than the count.
    #[must_use]
    pub fn get(self, index: usize) -> Option<Option<T::Ref<'a>>> {
        if index < self.len() {
            Some(unsafe { self.get_unchecked(index) })
        } else {
            None
        }
    }

    /// Gets the element at the `index` if it is present, without checking the `index`.
    ///
    /// # Safety
    /// The `index` must be less than the count.
    #[must_use]
    pub unsafe fn get_unchecked(self, index: usize) -> Option<T::Ref<'a>> {
        let is_present: bool = if T::ZERO_IS_ABSENT {
            let element: &[u8] = unsafe { self.elements.element_bytes_unchecked(index) };
            element.iter().any(|byte| *byte != 0)
        } else {
            (unsafe { *self.bits.get_unchecked(index / 8) }) >> (index % 8) & 1 == 1
        };
        if is_present {
            Some(unsafe { self.elements.get_unchecked(index) })
        } else {
            None
        }
    }
}

impl<'a, T> OptionListRef<'a, T>
where
    T: Element,
{
    //! Elements

    /// Gets the iterator of the elements, each `None` if it is absent.
    pub fn iter(self) -> impl ExactSizeIterator<Item = Option<T::Ref<'a>>> {
        (0..self.len()).map(move |index| unsafe { self.get_unchecked(index) })
    }

    /// Decodes the list.
    #[must_use]
    pub fn decode(self) -> Vec<Option<T>> {
        self.iter().map(|element| element.map(T::decode)).collect()
    }
}

impl<T> Debug for OptionListRef<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptionListRef")
            .field("bytes", &self.bytes)
            .finish()
    }
}

impl<T> Element for Vec<Option<T>>
where
    T: Element,
{
    const SIZE: Option<usize> = None;
    const ALIGNMENT: usize = option_list_alignment::<T>();
    const CAN_BE_INVALID: bool = true;
    const ORDERED_BY_BYTES: bool = T::ORDERED_BY_BYTES;
    const JSON_TEXT_KEY: bool = false;
    type Ref<'a> = OptionListRef<'a, T>;
    type View<'a>
        = &'a [Option<T>]
    where
        T: 'a;

    fn check(bytes: &[u8], depth: usize) -> Result<(), DecodeError> {
        OptionListRef::<T>::new_with_depth(bytes, depth).map(|_| ())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> OptionListRef<'_, T> {
        unsafe { OptionListRef::new_unchecked(bytes) }
    }

    fn decode(element: OptionListRef<'_, T>) -> Vec<Option<T>> {
        element.decode()
    }

    fn view(&self) -> &[Option<T>] {
        self
    }

    fn encoded_len(&self) -> usize {
        option_list_encoded_len(self)
    }

    fn encode_to(&self, out: &mut Vec<u8>, depth: usize) {
        encode_option_list(self, out, depth);
    }

    fn cmp_refs(a: OptionListRef<'_, T>, b: OptionListRef<'_, T>) -> Ordering
    where
        Self: Ord,
    {
        if T::ORDERED_BY_BYTES {
            a.as_bytes().cmp(b.as_bytes())
        } else {
            Self::decode(a).cmp(&Self::decode(b))
        }
    }

    #[cfg(feature = "types")]
    fn serialize_json<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::types::serialize_option_list(self, serializer)
    }

    #[cfg(feature = "types")]
    fn deserialize_json<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::types::deserialize_option_list(deserializer)
    }
}

impl<T> ArrayElement for Vec<Option<T>> where T: Element {}
