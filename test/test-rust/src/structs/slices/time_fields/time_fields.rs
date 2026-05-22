/// ```pps
/// // A struct with time slice fields.
/// struct TimeFields {
///    
///     // A `timestamp` slice field.
///     one: []timestamp;
///    
///     // A `date` slice field.
///     two: []date;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct TimeFields {
    one: Vec<i64>,
    two: Vec<proto_packet::types::Date>,
}

impl TimeFields {
    //! Construction

    /// Creates a new [TimeFields].
    pub const fn new(one: Vec<i64>, two: Vec<proto_packet::types::Date>) -> Self {
        Self { one, two }
    }

    /// Creates a new [TimeFields].
    pub fn from<F1, F2>(one: F1, two: F2) -> Self where F1: Into<Vec<i64>>, F2: Into<Vec<proto_packet::types::Date>> {
        Self::new(one.into(), two.into())
    }
}

impl TimeFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> &Vec<i64> {
        &self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Vec<i64>) -> Vec<i64> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Vec<i64>) -> Self {
        self.set_one(one);
        self
    }
}

impl TimeFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> &Vec<proto_packet::types::Date> {
        &self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Vec<proto_packet::types::Date>) -> Vec<proto_packet::types::Date> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Vec<proto_packet::types::Date>) -> Self {
        self.set_two(two);
        self
    }
}
