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
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Default,
    proto_packet::serde::Serialize,
    proto_packet::serde::Deserialize,
)]
#[serde(crate = "proto_packet::serde")]
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
    pub fn set_one<F>(&mut self, one: F) -> Option<Vec<proto_packet::types::Float32>>
    where
        F: Into<Option<Vec<proto_packet::types::Float32>>>,
    {
        let one: Option<Vec<proto_packet::types::Float32>> = one.into();
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one<F>(mut self, one: F) -> Self
    where
        F: Into<Option<Vec<proto_packet::types::Float32>>>,
    {
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
    pub fn set_two<F>(&mut self, two: F) -> Option<Vec<proto_packet::types::Float64>>
    where
        F: Into<Option<Vec<proto_packet::types::Float64>>>,
    {
        let two: Option<Vec<proto_packet::types::Float64>> = two.into();
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two<F>(mut self, two: F) -> Self
    where
        F: Into<Option<Vec<proto_packet::types::Float64>>>,
    {
        self.set_two(two);
        self
    }
}
