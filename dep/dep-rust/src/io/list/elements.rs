//! The elements that the runtime has.

use crate::io::list::element::list_alignment;
use crate::io::{DecodeError, Element, ListRef, check_bool, check_utf8, encode_list};
use crate::io::{list_encoded_len, read_bytes_unchecked};
use std::cmp::Ordering;

/// Implements an integer type as an element, with its alignment.
macro_rules! impl_integer {
    ($type:ty, $alignment:expr) => {
        impl Element for $type {
            const SIZE: Option<usize> = Some(size_of::<$type>());
            const ALIGNMENT: usize = $alignment;
            const CAN_BE_INVALID: bool = false;
            type Ref<'a> = $type;

            fn check(_bytes: &[u8]) -> Result<(), DecodeError> {
                Ok(())
            }

            unsafe fn read_unchecked(bytes: &[u8]) -> $type {
                <$type>::from_le_bytes(unsafe { read_bytes_unchecked(bytes, 0) })
            }

            fn decode(element: $type) -> $type {
                element
            }

            fn encoded_len(&self) -> usize {
                size_of::<$type>()
            }

            fn encode_to(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_le_bytes());
            }

            fn cmp_refs(a: $type, b: $type) -> Ordering {
                a.cmp(&b)
            }
        }
    };
}

impl_integer!(u16, 2);
impl_integer!(u32, 4);
impl_integer!(u64, 8);
impl_integer!(u128, 8);
impl_integer!(i8, 1);
impl_integer!(i16, 2);
impl_integer!(i32, 4);
impl_integer!(i64, 8);
impl_integer!(i128, 8);

impl Element for u8 {
    const SIZE: Option<usize> = Some(1);
    const ALIGNMENT: usize = 1;
    const CAN_BE_INVALID: bool = false;
    type Ref<'a> = u8;

    fn check(_bytes: &[u8]) -> Result<(), DecodeError> {
        Ok(())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> u8 {
        unsafe { *bytes.get_unchecked(0) }
    }

    fn decode(element: u8) -> u8 {
        element
    }

    fn encoded_len(&self) -> usize {
        1
    }

    fn encode_to(&self, out: &mut Vec<u8>) {
        out.push(*self);
    }

    fn encode_all_to(elements: &[u8], out: &mut Vec<u8>) {
        out.extend_from_slice(elements);
    }

    fn cmp_refs(a: u8, b: u8) -> Ordering {
        a.cmp(&b)
    }
}

impl Element for bool {
    const SIZE: Option<usize> = Some(1);
    const ALIGNMENT: usize = 1;
    const CAN_BE_INVALID: bool = true;
    type Ref<'a> = bool;

    fn check(bytes: &[u8]) -> Result<(), DecodeError> {
        check_bool(bytes[0])
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> bool {
        unsafe { *bytes.get_unchecked(0) != 0 }
    }

    fn decode(element: bool) -> bool {
        element
    }

    fn encoded_len(&self) -> usize {
        1
    }

    fn encode_to(&self, out: &mut Vec<u8>) {
        out.push(u8::from(*self));
    }

    fn cmp_refs(a: bool, b: bool) -> Ordering {
        a.cmp(&b)
    }
}

impl Element for String {
    const SIZE: Option<usize> = None;
    const ALIGNMENT: usize = 1;
    const CAN_BE_INVALID: bool = true;
    type Ref<'a> = &'a str;

    fn check(bytes: &[u8]) -> Result<(), DecodeError> {
        check_utf8(bytes)
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> &str {
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    fn decode(element: &str) -> String {
        element.to_string()
    }

    fn encoded_len(&self) -> usize {
        self.len()
    }

    fn encode_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.as_bytes());
    }

    fn cmp_refs(a: &str, b: &str) -> Ordering {
        a.cmp(b)
    }
}

impl<T> Element for Vec<T>
where
    T: Element,
{
    const SIZE: Option<usize> = None;
    const ALIGNMENT: usize = list_alignment::<T>();
    const CAN_BE_INVALID: bool = true;
    type Ref<'a> = ListRef<'a, T>;

    fn check(bytes: &[u8]) -> Result<(), DecodeError> {
        ListRef::<T>::new(bytes).map(|_| ())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> ListRef<'_, T> {
        unsafe { ListRef::new_unchecked(bytes) }
    }

    fn decode(element: ListRef<'_, T>) -> Vec<T> {
        element.decode()
    }

    fn encoded_len(&self) -> usize {
        list_encoded_len(self)
    }

    fn encode_to(&self, out: &mut Vec<u8>) {
        encode_list(self, out);
    }
}

/// Implements a float type as an element, with the integer type of its bits.
#[cfg(feature = "types")]
macro_rules! impl_float {
    ($type:ident, $bits:ty, $check:ident) => {
        impl Element for crate::types::$type {
            const SIZE: Option<usize> = Some(size_of::<$bits>());
            const ALIGNMENT: usize = size_of::<$bits>();
            const CAN_BE_INVALID: bool = true;
            type Ref<'a> = crate::types::$type;

            fn check(bytes: &[u8]) -> Result<(), DecodeError> {
                let bits: &[u8; size_of::<$bits>()] =
                    bytes.first_chunk().expect("a float of its size");
                crate::io::$check(<$bits>::from_le_bytes(*bits))
            }

            unsafe fn read_unchecked(bytes: &[u8]) -> crate::types::$type {
                let bits: $bits = <$bits>::from_le_bytes(unsafe { read_bytes_unchecked(bytes, 0) });
                unsafe { crate::types::$type::from_bits_unchecked(bits) }
            }

            fn decode(element: crate::types::$type) -> crate::types::$type {
                element
            }

            fn encoded_len(&self) -> usize {
                size_of::<$bits>()
            }

            fn encode_to(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_le_bytes());
            }

            fn cmp_refs(a: crate::types::$type, b: crate::types::$type) -> Ordering {
                a.cmp(&b)
            }
        }
    };
}

#[cfg(feature = "types")]
impl_float!(Float32, u32, check_f32);
#[cfg(feature = "types")]
impl_float!(Float64, u64, check_f64);

#[cfg(feature = "types")]
impl Element for crate::types::Uuid {
    const SIZE: Option<usize> = Some(16);
    const ALIGNMENT: usize = 8;
    const CAN_BE_INVALID: bool = false;
    type Ref<'a> = crate::types::Uuid;

    fn check(_bytes: &[u8]) -> Result<(), DecodeError> {
        Ok(())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> crate::types::Uuid {
        crate::types::Uuid::from_bytes(unsafe { read_bytes_unchecked(bytes, 0) })
    }

    fn decode(element: crate::types::Uuid) -> crate::types::Uuid {
        element
    }

    fn encoded_len(&self) -> usize {
        16
    }

    fn encode_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.as_bytes());
    }

    fn cmp_refs(a: crate::types::Uuid, b: crate::types::Uuid) -> Ordering {
        a.cmp(&b)
    }
}

/// Implements a time type as an element, with the integer type it holds.
#[cfg(feature = "types")]
macro_rules! impl_time {
    ($type:ident, $int:ty) => {
        impl Element for crate::types::$type {
            const SIZE: Option<usize> = Some(size_of::<$int>());
            const ALIGNMENT: usize = size_of::<$int>();
            const CAN_BE_INVALID: bool = false;
            type Ref<'a> = crate::types::$type;

            fn check(_bytes: &[u8]) -> Result<(), DecodeError> {
                Ok(())
            }

            unsafe fn read_unchecked(bytes: &[u8]) -> crate::types::$type {
                let value: $int = <$int>::from_le_bytes(unsafe { read_bytes_unchecked(bytes, 0) });
                crate::types::$type::new(value)
            }

            fn decode(element: crate::types::$type) -> crate::types::$type {
                element
            }

            fn encoded_len(&self) -> usize {
                size_of::<$int>()
            }

            fn encode_to(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_le_bytes());
            }

            fn cmp_refs(a: crate::types::$type, b: crate::types::$type) -> Ordering {
                a.cmp(&b)
            }
        }
    };
}

#[cfg(feature = "types")]
impl_time!(Timestamp, i64);
#[cfg(feature = "types")]
impl_time!(Microtime, i64);
#[cfg(feature = "types")]
impl_time!(Nanotime, i64);
#[cfg(feature = "types")]
impl_time!(Date, i32);
#[cfg(feature = "types")]
impl_time!(Duration, i64);

#[cfg(feature = "types")]
impl<const S: u8> Element for crate::types::Decimal<S> {
    const SIZE: Option<usize> = Some(8);
    const ALIGNMENT: usize = 8;
    const CAN_BE_INVALID: bool = false;
    type Ref<'a> = crate::types::Decimal<S>;

    fn check(_bytes: &[u8]) -> Result<(), DecodeError> {
        Ok(())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> crate::types::Decimal<S> {
        crate::types::Decimal::new(i64::from_le_bytes(unsafe {
            read_bytes_unchecked(bytes, 0)
        }))
    }

    fn decode(element: crate::types::Decimal<S>) -> crate::types::Decimal<S> {
        element
    }

    fn encoded_len(&self) -> usize {
        8
    }

    fn encode_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_le_bytes());
    }

    fn cmp_refs(a: crate::types::Decimal<S>, b: crate::types::Decimal<S>) -> Ordering {
        a.cmp(&b)
    }
}

#[cfg(feature = "types")]
impl Element for crate::types::Bytes {
    const SIZE: Option<usize> = None;
    const ALIGNMENT: usize = 1;
    const CAN_BE_INVALID: bool = false;
    type Ref<'a> = &'a [u8];

    fn check(_bytes: &[u8]) -> Result<(), DecodeError> {
        Ok(())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> &[u8] {
        bytes
    }

    fn decode(element: &[u8]) -> crate::types::Bytes {
        element.into()
    }

    fn encoded_len(&self) -> usize {
        self.len()
    }

    fn encode_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.as_bytes());
    }

    fn cmp_refs(a: &[u8], b: &[u8]) -> Ordering {
        a.cmp(b)
    }
}
