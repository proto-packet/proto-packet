/// A variant with special cases.
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
pub enum SpecialCases {
    /// A `uuid` case.
    One(proto_packet::types::Uuid),

    /// A `string` case.
    Two(String),
}
