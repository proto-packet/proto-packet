/// A variant with time cases.
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
pub enum TimeCases {
    /// A `timestamp` case.
    One(i64),

    /// A `date` case.
    Two(proto_packet::types::Date),
}
