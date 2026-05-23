/// A variant with a single boolean slice case.
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
pub enum BooleanCase {
    /// A `bool` slice case.
    One(Vec<bool>),
}
