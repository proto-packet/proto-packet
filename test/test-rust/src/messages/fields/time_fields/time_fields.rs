/// ```pps
/// // A message with time fields.
/// message TimeFields {
///    
///     // A `timestamp` field.
///     one: timestamp = 1;
///    
///     // A `date` field.
///     two: date = 2;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct TimeFields {
    one: Option<i64>,
    two: Option<proto_packet::types::Date>,
}

impl TimeFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<i64> {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<i64>) -> Option<i64> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<i64>) -> Self {
        self.set_one(one);
        self
    }
}

impl TimeFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<proto_packet::types::Date> {
        self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Option<proto_packet::types::Date>) -> Option<proto_packet::types::Date> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Option<proto_packet::types::Date>) -> Self {
        self.set_two(two);
        self
    }
}
