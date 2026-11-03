//! The helpers shared by the tests.

#![allow(dead_code)] // each test crate uses only some of the helpers

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
