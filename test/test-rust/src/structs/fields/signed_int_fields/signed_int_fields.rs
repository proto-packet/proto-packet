/// ```pps
/// // A struct with signed integer fields.
/// struct SignedIntFields {
///    
///     // An `i8` field.
///     one: i8;
///    
///     // An `i16` field.
///     two: i16;
///    
///     // An `i32` field.
///     three: i32;
///    
///     // An `i64` field.
///     four: i64;
///    
///     // An `i128` field.
///     five: i128;
/// }
/// ```
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct SignedIntFields {
    one: i8,
    two: i16,
    three: i32,
    four: i64,
    five: i128,
}

impl SignedIntFields {
    //! Construction

    /// Creates a new [SignedIntFields].
    pub const fn new(one: i8, two: i16, three: i32, four: i64, five: i128) -> Self {
        Self { one, two, three, four, five }
    }

    /// Creates a new [SignedIntFields].
    pub fn from<F1, F2, F3, F4, F5>(one: F1, two: F2, three: F3, four: F4, five: F5) -> Self where F1: Into<i8>, F2: Into<i16>, F3: Into<i32>, F4: Into<i64>, F5: Into<i128> {
        Self::new(one.into(), two.into(), three.into(), four.into(), five.into())
    }
}

impl SignedIntFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> i8 {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: i8) -> i8 {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: i8) -> Self {
        self.set_one(one);
        self
    }
}

impl SignedIntFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> i16 {
        self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: i16) -> i16 {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: i16) -> Self {
        self.set_two(two);
        self
    }
}

impl SignedIntFields {
    //! Field: `three`

    /// Gets the field: `three`.
    #[must_use]
    pub fn three(&self) -> i32 {
        self.three
    }

    /// Sets the field: `three`. Returns the previous value.
    pub fn set_three(&mut self, three: i32) -> i32 {
        std::mem::replace(&mut self.three, three)
    }

    /// Sets the field: `three`. Returns the struct itself.
    #[must_use]
    pub fn with_three(mut self, three: i32) -> Self {
        self.set_three(three);
        self
    }
}

impl SignedIntFields {
    //! Field: `four`

    /// Gets the field: `four`.
    #[must_use]
    pub fn four(&self) -> i64 {
        self.four
    }

    /// Sets the field: `four`. Returns the previous value.
    pub fn set_four(&mut self, four: i64) -> i64 {
        std::mem::replace(&mut self.four, four)
    }

    /// Sets the field: `four`. Returns the struct itself.
    #[must_use]
    pub fn with_four(mut self, four: i64) -> Self {
        self.set_four(four);
        self
    }
}

impl SignedIntFields {
    //! Field: `five`

    /// Gets the field: `five`.
    #[must_use]
    pub fn five(&self) -> i128 {
        self.five
    }

    /// Sets the field: `five`. Returns the previous value.
    pub fn set_five(&mut self, five: i128) -> i128 {
        std::mem::replace(&mut self.five, five)
    }

    /// Sets the field: `five`. Returns the struct itself.
    #[must_use]
    pub fn with_five(mut self, five: i128) -> Self {
        self.set_five(five);
        self
    }
}
