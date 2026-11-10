use crate::io::{DecodeError, JsonValue};
use std::cmp::Ordering;

/// A type that can be an element of a list.
///
/// # Note
/// - The bytes of a fixed-size element are exactly its size.
pub trait Element: Sized + JsonValue {
    /// The size of every encoded element, or `None` for a variable-size type.
    const SIZE: Option<usize>;

    /// The alignment of an encoded element.
    const ALIGNMENT: usize;

    /// If the bytes of an element can be invalid. (ex: false for an integer of the right size)
    const CAN_BE_INVALID: bool;

    /// If a key of the type is ordered by its encoded bytes in a map.
    ///
    /// # Note
    /// - A message or a variant is, & so is a type with one inside it.
    /// - Two readers can know different fields & cases of one, so its bytes are the only order
    ///   they share.
    const ORDERED_BY_BYTES: bool = false;

    /// If a key of the type is text in JSON, so a map of it is an object.
    ///
    /// # Note
    /// - A map of any other key is a list of its entries.
    const JSON_TEXT_KEY: bool = true;

    /// The element read from its encoded bytes.
    type Ref<'a>: Copy;

    /// The element borrowed from the type itself. (ex: `&str` for a `String`)
    ///
    /// # Note
    /// - It is the `Ref` type where it can be, so a list reads the same in every form.
    type View<'a>
    where
        Self: 'a;

    /// Checks the `bytes` are a valid element, which can nest the `depth` of levels.
    ///
    /// # Errors
    /// Returns an error if the `bytes` are not a valid element.
    fn check(bytes: &[u8], depth: usize) -> Result<(), DecodeError>;

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

    /// Appends the encoding to the `out` bytes, which can nest the `depth` of levels.
    ///
    /// # Panics
    /// Panics if the element nests deeper.
    fn encode_to(&self, out: &mut Vec<u8>, depth: usize);

    /// Compares two read elements, in the order of the keys of a map.
    ///
    /// # Note
    /// - A type overrides this when it can compare the elements without decoding them.
    /// - A type that is ordered by its bytes overrides this to compare them.
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
        if Self::ORDERED_BY_BYTES {
            let mut bytes: Vec<u8> = Vec::with_capacity(self.encoded_len());
            self.encode_to(&mut bytes, usize::MAX);
            Self::cmp_refs(unsafe { Self::read_unchecked(&bytes) }, other)
        } else {
            self.cmp(&Self::decode(other))
        }
    }

    /// Serializes the element as its JSON form.
    ///
    /// # Note
    /// - A list or a map overrides this, since its JSON form is not the one serde gives it.
    ///
    /// # Errors
    /// Returns an error if the `serializer` does.
    #[cfg(feature = "types")]
    fn serialize_json<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }

    /// Deserializes the element from its JSON form.
    ///
    /// # Errors
    /// Returns an error if the JSON is not the element.
    #[cfg(feature = "types")]
    fn deserialize_json<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }

    /// Appends the encodings of the fixed-size `elements` to the `out` bytes, back to back.
    ///
    /// # Note
    /// - A type overrides this when its elements can be copied in one block. (ex: `u8`)
    /// - A fixed-size element is not a level, so it needs no depth.
    fn encode_all_to(elements: &[Self], out: &mut Vec<u8>) {
        for element in elements {
            element.encode_to(out, 0);
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
