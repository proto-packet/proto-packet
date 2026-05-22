/// ```pps
/// // A struct with special fields.
/// struct SpecialFields {
///    
///     // A `uuid` field.
///     one: uuid;
/// }
/// ```
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct SpecialFields {
    one: proto_packet::types::Uuid,
}

impl SpecialFields {
    //! Construction

    /// Creates a new [SpecialFields].
    pub const fn new(one: proto_packet::types::Uuid) -> Self {
        Self { one }
    }

    /// Creates a new [SpecialFields].
    pub fn from<F1>(one: F1) -> Self where F1: Into<proto_packet::types::Uuid> {
        Self::new(one.into())
    }
}

impl SpecialFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> proto_packet::types::Uuid {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: proto_packet::types::Uuid) -> proto_packet::types::Uuid {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: proto_packet::types::Uuid) -> Self {
        self.set_one(one);
        self
    }
}
