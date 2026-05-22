/// A variant with signed integer cases.
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
pub enum SignedIntCases {
    /// An `i8` case.
    One(i8),

    /// An `i16` case.
    Two(i16),

    /// An `i32` case.
    Three(i32),

    /// An `i64` case.
    Four(i64),

    /// An `i128` case.
    Five(i128),
}
