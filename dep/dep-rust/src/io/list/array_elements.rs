//! The arrays as elements.
//!
//! An array is a list of exactly its length of fixed-size elements, so it is fixed-size & has no
//! count. An array of optional elements has their presence bits after them. An array of bytes is
//! read in place as its bytes.

use crate::io::list::element::stride;
use crate::io::list::option_write::{bits_len, option_items_len, write_option_items};
use crate::io::{ArrayElement, DecodeError, Element, ListRef, OptionListRef};
use crate::io::{check_exact_len, encode_list};
use std::cmp::Ordering;

/// Gets the size of an array of the `len` of elements of the type `T`, if they are fixed-size.
const fn array_size<T>(len: usize) -> Option<usize>
where
    T: Element,
{
    match T::SIZE {
        Some(size) if len > 0 => Some((len - 1) * stride::<T>(size) + size),
        Some(_) => Some(0),
        None => None,
    }
}

/// Gets the size of an array of the `len` of optional elements of the type `T`, if they are
/// fixed-size.
const fn option_array_size<T>(len: usize) -> Option<usize>
where
    T: Element,
{
    match array_size::<T>(len) {
        Some(size) => Some(size + bits_len::<T>(len)),
        None => None,
    }
}

/// Checks the `bytes` are the `size` of an array, which has fixed-size elements.
fn check_array_len(bytes: &[u8], size: Option<usize>) -> Result<(), DecodeError> {
    match size {
        Some(size) => check_exact_len(bytes, size),
        None => Err(DecodeError::InvalidList),
    }
}

impl<T, const N: usize> ArrayElement for [T; N] where T: ArrayElement {}

impl<T, const N: usize> ArrayElement for [Option<T>; N] where T: Element {}

impl<const N: usize> ArrayElement for [u8; N] {}

impl<const N: usize> Element for [u8; N] {
    const SIZE: Option<usize> = Some(N);
    const ALIGNMENT: usize = 1;
    const CAN_BE_INVALID: bool = false;
    type Ref<'a> = &'a [u8; N];
    type View<'a> = &'a [u8; N];

    fn check(bytes: &[u8], _depth: usize) -> Result<(), DecodeError> {
        check_exact_len(bytes, N)
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> &[u8; N] {
        unsafe { <&[u8; N]>::try_from(bytes).unwrap_unchecked() }
    }

    fn decode(element: &[u8; N]) -> [u8; N] {
        *element
    }

    fn view(&self) -> &[u8; N] {
        self
    }

    fn encoded_len(&self) -> usize {
        N
    }

    fn encode_to(&self, out: &mut Vec<u8>, _depth: usize) {
        out.extend_from_slice(self);
    }

    fn cmp_refs(a: &[u8; N], b: &[u8; N]) -> Ordering {
        a.cmp(b)
    }

    fn cmp_ref(&self, other: &[u8; N]) -> Ordering {
        self.cmp(other)
    }

    #[cfg(feature = "types")]
    fn serialize_json<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::types::serialize_byte_array(self, serializer)
    }

    #[cfg(feature = "types")]
    fn deserialize_json<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::types::deserialize_byte_array(deserializer)
    }
}

impl<T, const N: usize> Element for [T; N]
where
    T: ArrayElement,
{
    const SIZE: Option<usize> = array_size::<T>(N);
    const ALIGNMENT: usize = T::ALIGNMENT;
    const CAN_BE_INVALID: bool = true;
    const ORDERED_BY_BYTES: bool = T::ORDERED_BY_BYTES;
    const JSON_TEXT_KEY: bool = false;
    type Ref<'a> = ListRef<'a, T>;
    type View<'a>
        = &'a [T]
    where
        T: 'a;

    fn check(bytes: &[u8], depth: usize) -> Result<(), DecodeError> {
        check_array_len(bytes, Self::SIZE)?;
        ListRef::<T>::new_with_depth(bytes, depth).map(|_| ())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> ListRef<'_, T> {
        unsafe { ListRef::new_unchecked(bytes) }
    }

    fn decode(element: ListRef<'_, T>) -> [T; N] {
        std::array::from_fn(|index| T::decode(unsafe { element.get_unchecked(index) }))
    }

    fn view(&self) -> &[T] {
        self
    }

    fn encoded_len(&self) -> usize {
        Self::SIZE.unwrap_or_default()
    }

    fn encode_to(&self, out: &mut Vec<u8>, depth: usize) {
        encode_list(self, out, depth);
    }

    #[cfg(feature = "types")]
    fn serialize_json<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::types::serialize_array(self, serializer)
    }

    #[cfg(feature = "types")]
    fn deserialize_json<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::types::deserialize_array(deserializer)
    }
}

impl<T, const N: usize> Element for [Option<T>; N]
where
    T: Element,
{
    const SIZE: Option<usize> = option_array_size::<T>(N);
    const ALIGNMENT: usize = T::ALIGNMENT;
    const CAN_BE_INVALID: bool = true;
    const ORDERED_BY_BYTES: bool = T::ORDERED_BY_BYTES;
    const JSON_TEXT_KEY: bool = false;
    type Ref<'a> = OptionListRef<'a, T>;
    type View<'a>
        = &'a [Option<T>]
    where
        T: 'a;

    fn check(bytes: &[u8], depth: usize) -> Result<(), DecodeError> {
        check_array_len(bytes, Self::SIZE)?;
        OptionListRef::<T>::of_len_with_depth(bytes, N, depth).map(|_| ())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> OptionListRef<'_, T> {
        unsafe { OptionListRef::of_len_unchecked(bytes, N) }
    }

    fn decode(element: OptionListRef<'_, T>) -> [Option<T>; N] {
        std::array::from_fn(|index| unsafe { element.get_unchecked(index) }.map(T::decode))
    }

    fn view(&self) -> &[Option<T>] {
        self
    }

    fn encoded_len(&self) -> usize {
        option_items_len::<T, _, _, T, _>(self.iter().map(Option::as_ref), N, &T::encoded_len)
    }

    fn encode_to(&self, out: &mut Vec<u8>, depth: usize) {
        write_option_items::<T, _, _, T, _>(
            self.iter().map(Option::as_ref),
            N,
            out,
            &mut |item, out| item.encode_to(out, depth),
        );
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
        crate::types::deserialize_option_array(deserializer)
    }
}
