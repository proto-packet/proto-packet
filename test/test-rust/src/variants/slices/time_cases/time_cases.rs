/// A variant with time slice cases.
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
    /// A `timestamp` slice case.
    One(Vec<i64>),

    /// A `date` slice case.
    Two(Vec<proto_packet::types::Date>),
}
