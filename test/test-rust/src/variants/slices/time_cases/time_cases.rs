/// A variant with time slice cases.
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
pub enum TimeCases {
    /// A `timestamp` slice case.
    One(Vec<i64>),

    /// A `date` slice case.
    Two(Vec<proto_packet::types::Date>),
}
