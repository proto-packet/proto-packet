//! The helpers shared by the tests.

#![allow(dead_code, unused_macros, unused_imports)] // each test crate uses only some of the helpers

use proto_packet::serde::Serialize;
use proto_packet::serde::de::DeserializeOwned;
use proto_packet::types::Uuid;
use std::fmt::Debug;

/// Asserts the `value` serializes to the `json` & deserializes from it.
pub fn assert_json<T>(value: &T, json: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    assert_eq!(serde_json::to_string(value).unwrap(), json);
    assert_eq!(&serde_json::from_str::<T>(json).unwrap(), value);
}

/// The test UUID.
pub const ID: Uuid = Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);

/// The JSON of the test UUID.
pub const ID_JSON: &str = r#""01234567-89ab-cdef-0123-456789abcdef""#;

/// Asserts the valid `bytes` are the only encoding of their value.
///
/// Every change to the `bytes` must fail to decode, or encode back to the changed bytes. The
/// `recode` function decodes its bytes & encodes them again, or gives `None` if they are invalid.
///
/// # Note
/// - The changes are a bit, a byte, a cut, a removed or added byte, & a few of these at once.
/// - A moved offset with the padding it needs is tried for every pair of places in short bytes.
pub fn assert_one_encoding<F>(bytes: &[u8], recode: F)
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    assert_eq!(recode(bytes).as_deref(), Some(bytes));
    let check = |changed: &[u8]| {
        if let Some(encoded) = recode(changed) {
            assert_eq!(
                encoded, changed,
                "the changed bytes decode, but their value encodes to other bytes, from {:?}",
                bytes
            );
        }
    };
    for index in 0..bytes.len() {
        let bits = (0..8).map(|bit| bytes[index] ^ (1 << bit));
        for value in bits.chain([0, 1, 2, u8::MAX]) {
            let mut changed: Vec<u8> = bytes.to_vec();
            changed[index] = value;
            check(&changed);
        }
        check(&bytes[..index]);
        let mut changed: Vec<u8> = bytes.to_vec();
        changed.remove(index);
        check(&changed);
    }
    for index in 0..=bytes.len() {
        for value in [0, 1] {
            let mut changed: Vec<u8> = bytes.to_vec();
            changed.insert(index, value);
            check(&changed);
        }
    }
    if bytes.len() <= 48 {
        for (offset, at) in (0..bytes.len()).flat_map(|a| (0..=bytes.len()).map(move |b| (a, b))) {
            for count in [1_u8, 2, 3, 4, 7] {
                let mut changed: Vec<u8> = bytes.to_vec();
                changed[offset] = changed[offset].wrapping_add(count);
                let zeros = std::iter::repeat_n(0, usize::from(count));
                changed.splice(at..at, zeros);
                check(&changed);

                let end: usize = at + usize::from(count);
                if bytes
                    .get(at..end)
                    .is_some_and(|gap| gap.iter().all(|byte| *byte == 0))
                {
                    let mut changed: Vec<u8> = bytes.to_vec();
                    changed[offset] = changed[offset].wrapping_sub(count);
                    changed.drain(at..end);
                    check(&changed);
                }
            }
        }
    }
    let mut state: u64 = bytes.iter().fold(0x9E37_79B9_7F4A_7C15, |state, byte| {
        (state ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01B3)
    }) | 1;
    let mut next = move || -> usize {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 16) as usize
    };
    for _ in 0..2000 {
        let mut changed: Vec<u8> = bytes.to_vec();
        for _ in 0..=next() % 3 {
            if changed.is_empty() {
                changed.push(next() as u8);
                continue;
            }
            let index: usize = next() % changed.len();
            match next() % 5 {
                0 => changed[index] = [0, 1, 2, 3, 4, 8, u8::MAX][next() % 7],
                1 => changed[index] = changed[index].wrapping_add(1 + (next() % 8) as u8),
                2 => changed[index] = changed[index].wrapping_sub(1 + (next() % 8) as u8),
                3 => changed.insert(index, [0, 0, 0, 1][next() % 4]),
                _ => drop(changed.remove(index)),
            }
        }
        check(&changed);
    }
}

/// A value with only the getters of its `View` trait.
///
/// # Note
/// - It overrides nothing, so it is encoded by the provided functions of the trait.
pub struct Plain<'a, T> {
    pub value: &'a T,
}

/// A type that is encoded from only the getters of its `View` trait.
pub trait ViewEncode {
    /// Gets the encoded length & the encoding, from only the getters.
    fn view_encode(&self) -> (usize, Vec<u8>);
}

/// Implements the `View` trait of a type for its [Plain] value, from the getters of the trait.
macro_rules! plain {
    ($view:path, $type:ty, { $(fn $getter:ident(&self $(,)?) -> $result:ty;)* }) => {
        impl $view for common::Plain<'_, $type> {
            $(
                fn $getter(&self) -> $result {
                    self.value.$getter()
                }
            )*
        }

        impl common::ViewEncode for $type {
            fn view_encode(&self) -> (usize, Vec<u8>) {
                let plain: common::Plain<'_, $type> = common::Plain { value: self };
                (
                    <common::Plain<'_, $type> as $view>::encoded_len(&plain),
                    <common::Plain<'_, $type> as $view>::encode(&plain),
                )
            }
        }
    };
}
pub(crate) use plain;
