/// ```pps
/// // A message with time slice fields.
/// message TimeFields {
///    
///     // A `timestamp` slice field.
///     one: []timestamp = 1;
///    
///     // A `date` slice field.
///     two: []date = 2;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct TimeFields {
    one: Option<Vec<i64>>,
    two: Option<Vec<proto_packet::types::Date>>,
}

impl TimeFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<&Vec<i64>> {
        self.one.as_ref()
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<Vec<i64>>) -> Option<Vec<i64>> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<Vec<i64>>) -> Self {
        self.set_one(one);
        self
    }
}

impl TimeFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<&Vec<proto_packet::types::Date>> {
        self.two.as_ref()
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Option<Vec<proto_packet::types::Date>>) -> Option<Vec<proto_packet::types::Date>> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Option<Vec<proto_packet::types::Date>>) -> Self {
        self.set_two(two);
        self
    }
}
