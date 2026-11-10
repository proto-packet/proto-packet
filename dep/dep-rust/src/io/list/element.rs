use crate::io::DecodeError;
use std::cmp::Ordering;

/// A type that can be an element of a list.
///
/// # Note
/// - The bytes of a fixed-size element are exactly its size.
pub trait Element: Sized {
    /// The size of every encoded element, or `None` for a variable-size type.
    const SIZE: Option<usize>;

    /// The alignment of an encoded element.
    const ALIGNMENT: usize;

    /// If the bytes of an element can be invalid. (ex: false for an integer of the right size)
    const CAN_BE_INVALID: bool;

    /// The element read from its encoded bytes.
    type Ref<'a>: Copy;

    /// The element borrowed from the type itself. (ex: `&str` for a `String`)
    ///
    /// # Note
    /// - It is the `Ref` type where it can be, so a list reads the same in every form.
    type View<'a>
    where
        Self: 'a;

    /// Checks the `bytes` are a valid element.
    ///
    /// # Errors
    /// Returns an error if the `bytes` are not a valid element.
    fn check(bytes: &[u8]) -> Result<(), DecodeError>;

    /// Reads the element from its `bytes`.
    ///
    /// # Safety
    /// The `bytes` must be a valid element.
    unsafe fn read_unchecked(bytes: &[u8]) -> Self::Ref<'_>;

    /// Decodes the read `element`.
    fn decode(element: Self::Ref<'_>) -> Self;

    /// Borrows the element.
    fn view(&self) -> Self::View<'_>;

    /// Gets the length of the encoding, in bytes.
    fn encoded_len(&self) -> usize;

    /// Appends the encoding to the `out` bytes.
    fn encode_to(&self, out: &mut Vec<u8>);

    /// Compares two read elements, in the order of the keys of a map.
    ///
    /// # Note
    /// - A type overrides this when it can compare the elements without decoding them.
    fn cmp_refs(a: Self::Ref<'_>, b: Self::Ref<'_>) -> Ordering
    where
        Self: Ord,
    {
        Self::decode(a).cmp(&Self::decode(b))
    }

    /// Compares the element to a read element, in the order of the keys of a map.
    ///
    /// # Note
    /// - A type overrides this when it can compare without decoding the read element.
    fn cmp_ref(&self, other: Self::Ref<'_>) -> Ordering
    where
        Self: Ord,
    {
        self.cmp(&Self::decode(other))
    }

    /// Appends the encodings of the fixed-size `elements` to the `out` bytes, back to back.
    ///
    /// # Note
    /// - A type overrides this when its elements can be copied in one block. (ex: `u8`)
    fn encode_all_to(elements: &[Self], out: &mut Vec<u8>) {
        for element in elements {
            element.encode_to(out);
        }
    }
}

/// Gets the size of a fixed-size element with its padding to the next element.
pub(crate) const fn stride<T>(size: usize) -> usize
where
    T: Element,
{
    crate::io::align_up(size, T::ALIGNMENT)
}

/// Gets the alignment of a list of the element `T`.
///
/// # Note
/// - A list of variable-size elements has a count & an end table, of alignment 4.
#[must_use]
pub const fn list_alignment<T>() -> usize
where
    T: Element,
{
    if T::SIZE.is_some() || T::ALIGNMENT > 4 {
        T::ALIGNMENT
    } else {
        4
    }
}
