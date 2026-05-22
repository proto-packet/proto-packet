/// ```pps
/// // A struct with a single boolean slice field.
/// struct BooleanField {
///    
///     // A `bool` slice field.
///     one: []bool;
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
    proto_packet::serde::Serialize,
    proto_packet::serde::Deserialize,
)]
#[serde(crate = "proto_packet::serde")]
pub struct BooleanField {
    one: Vec<bool>,
}

impl BooleanField {
    //! Construction

    /// Creates a new [BooleanField].
    pub const fn new(one: Vec<bool>) -> Self {
        Self { one }
    }

    /// Creates a new [BooleanField].
    pub fn from<F1>(one: F1) -> Self
    where
        F1: Into<Vec<bool>>,
    {
        Self::new(one.into())
    }
}

impl BooleanField {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> &Vec<bool> {
        &self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Vec<bool>) -> Vec<bool> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Vec<bool>) -> Self {
        self.set_one(one);
        self
    }
}
