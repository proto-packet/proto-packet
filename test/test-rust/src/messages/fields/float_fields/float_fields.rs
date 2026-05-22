/// ```pps
/// // A message with floating point fields.
/// message FloatFields {
///    
///     // An `f32` field.
///     one: f32 = 1;
///    
///     // An `f64` field.
///     two: f64 = 2;
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
    serde::Serialize,
    serde::Deserialize,
)]
pub struct FloatFields {
    one: Option<proto_packet::types::Float32>,
    two: Option<proto_packet::types::Float64>,
}

impl FloatFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<f32> {
        self.one.map(Into::into)
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<f32>) -> Option<f32> {
        std::mem::replace(&mut self.one, one.map(Into::into)).map(Into::into)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<f32>) -> Self {
        self.set_one(one);
        self
    }
}

impl FloatFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<f64> {
        self.two.map(Into::into)
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Option<f64>) -> Option<f64> {
        std::mem::replace(&mut self.two, two.map(Into::into)).map(Into::into)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Option<f64>) -> Self {
        self.set_two(two);
        self
    }
}
