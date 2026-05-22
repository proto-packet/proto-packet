/// A variant with floating point cases.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum FloatCases {
    /// An `f32` case.
    One(proto_packet::types::Float32),
    
    /// An `f64` case.
    Two(proto_packet::types::Float64),
}
