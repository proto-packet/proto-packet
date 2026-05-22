/// ```pps
/// // A struct with a single boolean field.
/// struct BooleanField {
///    
///     // A `bool` field.
///     one: bool;
/// }
/// ```
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct BooleanField {
    one: bool,
}

impl BooleanField {
    //! Construction

    /// Creates a new [BooleanField].
    pub const fn new(one: bool) -> Self {
        Self { one }
    }

    /// Creates a new [BooleanField].
    pub fn from<F1>(one: F1) -> Self where F1: Into<bool> {
        Self::new(one.into())
    }
}

impl BooleanField {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> bool {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: bool) -> bool {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: bool) -> Self {
        self.set_one(one);
        self
    }
}
