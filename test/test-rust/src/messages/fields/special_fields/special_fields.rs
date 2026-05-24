/// ```pps
/// // A message with special fields.
/// message SpecialFields {
///    
///     // A `uuid` field.
///     one: uuid = 1;
///    
///     // A `string` field.
///     two: string = 2;
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
pub struct SpecialFields {
    one: Option<proto_packet::types::Uuid>,
    two: Option<String>,
}

impl SpecialFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<proto_packet::types::Uuid> {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one<F>(&mut self, one: F) -> Option<proto_packet::types::Uuid>
    where
        F: Into<Option<proto_packet::types::Uuid>>,
    {
        let one: Option<proto_packet::types::Uuid> = one.into();
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one<F>(mut self, one: F) -> Self
    where
        F: Into<Option<proto_packet::types::Uuid>>,
    {
        self.set_one(one);
        self
    }
}

impl SpecialFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<&str> {
        self.two.as_deref()
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two<F>(&mut self, two: F) -> Option<String>
    where
        F: Into<Option<String>>,
    {
        let two: Option<String> = two.into();
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two<F>(mut self, two: F) -> Self
    where
        F: Into<Option<String>>,
    {
        self.set_two(two);
        self
    }
}
