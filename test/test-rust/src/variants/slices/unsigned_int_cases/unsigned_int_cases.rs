/// A variant with unsigned integer slice cases.
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    proto_packet::serde::Serialize,
    proto_packet::serde::Deserialize,
)]
#[serde(crate = "proto_packet::serde")]
#[serde(tag = "var", content = "val")]
pub enum UnsignedIntCases {
    /// A `u8` slice case.
    One(Vec<u8>),

    /// A `u16` slice case.
    Two(Vec<u16>),

    /// A `u32` slice case.
    Three(Vec<u32>),

    /// A `u64` slice case.
    Four(Vec<u64>),

    /// A `u128` slice case.
    Five(Vec<u128>),
}
