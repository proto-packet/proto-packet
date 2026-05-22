/// ```pps
/// // A message with time fields.
/// message TimeFields {
///    
///     // A `timestamp` field.
///     one: timestamp = 1;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct TimeFields {
    one: Option<i64>,
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
