/// ```pps
/// // A message with special slice fields.
/// message SpecialFields {
///    
///     // A `uuid` slice field.
///     one: []uuid = 1;
///    
///     // A `string` slice field.
///     two: []string = 2;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct SpecialFields {
    one: Option<Vec<proto_packet::types::Uuid>>,
    two: Option<Vec<String>>,
}

impl SpecialFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<&Vec<proto_packet::types::Uuid>> {
        self.one.as_ref()
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<Vec<proto_packet::types::Uuid>>) -> Option<Vec<proto_packet::types::Uuid>> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<Vec<proto_packet::types::Uuid>>) -> Self {
        self.set_one(one);
        self
    }
}

impl SpecialFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<&Vec<String>> {
        self.two.as_ref()
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Option<Vec<String>>) -> Option<Vec<String>> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Option<Vec<String>>) -> Self {
        self.set_two(two);
        self
    }
}
