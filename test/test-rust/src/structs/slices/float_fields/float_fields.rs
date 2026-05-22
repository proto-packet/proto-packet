/// ```pps
/// // A struct with floating point slice fields.
/// struct FloatFields {
///    
///     // An `f32` slice field.
///     one: []f32;
///    
///     // An `f64` slice field.
///     two: []f64;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct FloatFields {
    one: Vec<proto_packet::types::Float32>,
    two: Vec<proto_packet::types::Float64>,
}

impl FloatFields {
    //! Construction

    /// Creates a new [FloatFields].
    pub const fn new(
        one: Vec<proto_packet::types::Float32>,
        two: Vec<proto_packet::types::Float64>,
    ) -> Self {
        Self { one, two }
    }

    /// Creates a new [FloatFields].
    pub fn from<F1, F2>(one: F1, two: F2) -> Self
    where
        F1: Into<Vec<proto_packet::types::Float32>>,
        F2: Into<Vec<proto_packet::types::Float64>>,
    {
        Self::new(one.into(), two.into())
    }
}

impl FloatFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> &Vec<proto_packet::types::Float32> {
        &self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(
        &mut self,
        one: Vec<proto_packet::types::Float32>,
    ) -> Vec<proto_packet::types::Float32> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Vec<proto_packet::types::Float32>) -> Self {
        self.set_one(one);
        self
    }
}

impl FloatFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> &Vec<proto_packet::types::Float64> {
        &self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(
        &mut self,
        two: Vec<proto_packet::types::Float64>,
    ) -> Vec<proto_packet::types::Float64> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Vec<proto_packet::types::Float64>) -> Self {
        self.set_two(two);
        self
    }
}
