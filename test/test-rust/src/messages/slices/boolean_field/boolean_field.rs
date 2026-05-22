/// ```pps
/// // A message with a single boolean slice field.
/// message BooleanField {
///    
///     // A `bool` slice field.
///     one: []bool = 1;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct BooleanField {
    one: Option<Vec<bool>>,
}

impl BooleanField {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<&Vec<bool>> {
        self.one.as_ref()
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<Vec<bool>>) -> Option<Vec<bool>> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<Vec<bool>>) -> Self {
        self.set_one(one);
        self
    }
}
