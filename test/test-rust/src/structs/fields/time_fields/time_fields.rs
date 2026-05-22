/// ```pps
/// // A struct with time fields.
/// struct TimeFields {
///    
///     // A `timestamp` field.
///     one: timestamp;
/// }
/// ```
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct TimeFields {
    one: i64,
}

impl TimeFields {
    //! Construction

    /// Creates a new [TimeFields].
    pub const fn new(one: i64) -> Self {
        Self { one }
    }

    /// Creates a new [TimeFields].
    pub fn from<F1>(one: F1) -> Self where F1: Into<i64> {
        Self::new(one.into())
    }
}

impl TimeFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> i64 {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: i64) -> i64 {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: i64) -> Self {
        self.set_one(one);
        self
    }
}
