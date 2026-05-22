/// A variant with floating point slice cases.
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
pub enum FloatCases {
    /// An `f32` slice case.
    One(Vec<proto_packet::types::Float32>),

    /// An `f64` slice case.
    Two(Vec<proto_packet::types::Float64>),
}
