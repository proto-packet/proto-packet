/// A variant with time cases.
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
pub enum TimeCases {
    /// A `timestamp` case.
    One(i64),

    /// A `date` case.
    Two(proto_packet::types::Date),
}
