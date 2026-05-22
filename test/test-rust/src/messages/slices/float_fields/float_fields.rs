/// ```pps
/// // A message with floating point slice fields.
/// message FloatFields {
///    
///     // An `f32` slice field.
///     one: []f32 = 1;
///    
///     // An `f64` slice field.
///     two: []f64 = 2;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct FloatFields {
    one: Option<Vec<proto_packet::types::Float32>>,
    two: Option<Vec<proto_packet::types::Float64>>,
}

impl FloatFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<&Vec<proto_packet::types::Float32>> {
        self.one.as_ref()
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<Vec<proto_packet::types::Float32>>) -> Option<Vec<proto_packet::types::Float32>> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<Vec<proto_packet::types::Float32>>) -> Self {
        self.set_one(one);
        self
    }
}

impl FloatFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<&Vec<proto_packet::types::Float64>> {
        self.two.as_ref()
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Option<Vec<proto_packet::types::Float64>>) -> Option<Vec<proto_packet::types::Float64>> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Option<Vec<proto_packet::types::Float64>>) -> Self {
        self.set_two(two);
        self
    }
}
