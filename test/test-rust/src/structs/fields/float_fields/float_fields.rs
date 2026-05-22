/// ```pps
/// // A struct with floating point fields.
/// struct FloatFields {
///    
///     // An `f32` field.
///     one: f32;
///    
///     // An `f64` field.
///     two: f64;
/// }
/// ```
#[derive(
    Copy,
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
pub struct FloatFields {
    one: proto_packet::types::Float32,
    two: proto_packet::types::Float64,
}

impl FloatFields {
    //! Construction

    /// Creates a new [FloatFields].
    pub const fn new(one: f32, two: f64) -> Self {
        Self {
            one: proto_packet::types::Float32::new(one),
            two: proto_packet::types::Float64::new(two),
        }
    }

    /// Creates a new [FloatFields].
    pub fn from<F1, F2>(one: F1, two: F2) -> Self
    where
        F1: Into<f32>,
        F2: Into<f64>,
    {
        Self::new(one.into(), two.into())
    }
}

impl FloatFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> f32 {
        self.one.into()
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: f32) -> f32 {
        std::mem::replace(&mut self.one, one.into()).into()
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: f32) -> Self {
        self.set_one(one);
        self
    }
}

impl FloatFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> f64 {
        self.two.into()
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: f64) -> f64 {
        std::mem::replace(&mut self.two, two.into()).into()
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: f64) -> Self {
        self.set_two(two);
        self
    }
}
