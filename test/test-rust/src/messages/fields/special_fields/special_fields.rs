/// ```pps
/// // A message with special fields.
/// message SpecialFields {
///    
///     // A `uuid` field.
///     one: uuid = 1;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct SpecialFields {
    one: Option<proto_packet::types::Uuid>,
}

impl SpecialFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<proto_packet::types::Uuid> {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<proto_packet::types::Uuid>) -> Option<proto_packet::types::Uuid> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<proto_packet::types::Uuid>) -> Self {
        self.set_one(one);
        self
    }
}
