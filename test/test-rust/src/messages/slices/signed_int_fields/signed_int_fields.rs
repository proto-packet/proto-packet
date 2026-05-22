/// ```pps
/// // A message with signed integer slice fields.
/// message SignedIntFields {
///    
///     // An `i8` slice field.
///     one: []i8 = 1;
///    
///     // An `i16` slice field.
///     two: []i16 = 2;
///    
///     // An `i32` slice field.
///     three: []i32 = 3;
///    
///     // An `i64` slice field.
///     four: []i64 = 4;
///    
///     // An `i128` slice field.
///     five: []i128 = 5;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct SignedIntFields {
    one: Option<Vec<i8>>,
    two: Option<Vec<i16>>,
    three: Option<Vec<i32>>,
    four: Option<Vec<i64>>,
    five: Option<Vec<i128>>,
}

impl SignedIntFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<&Vec<i8>> {
        self.one.as_ref()
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<Vec<i8>>) -> Option<Vec<i8>> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<Vec<i8>>) -> Self {
        self.set_one(one);
        self
    }
}

impl SignedIntFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<&Vec<i16>> {
        self.two.as_ref()
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Option<Vec<i16>>) -> Option<Vec<i16>> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Option<Vec<i16>>) -> Self {
        self.set_two(two);
        self
    }
}

impl SignedIntFields {
    //! Field: `three`

    /// Gets the field: `three`.
    #[must_use]
    pub fn three(&self) -> Option<&Vec<i32>> {
        self.three.as_ref()
    }

    /// Sets the field: `three`. Returns the previous value.
    pub fn set_three(&mut self, three: Option<Vec<i32>>) -> Option<Vec<i32>> {
        std::mem::replace(&mut self.three, three)
    }

    /// Sets the field: `three`. Returns the struct itself.
    #[must_use]
    pub fn with_three(mut self, three: Option<Vec<i32>>) -> Self {
        self.set_three(three);
        self
    }
}

impl SignedIntFields {
    //! Field: `four`

    /// Gets the field: `four`.
    #[must_use]
    pub fn four(&self) -> Option<&Vec<i64>> {
        self.four.as_ref()
    }

    /// Sets the field: `four`. Returns the previous value.
    pub fn set_four(&mut self, four: Option<Vec<i64>>) -> Option<Vec<i64>> {
        std::mem::replace(&mut self.four, four)
    }

    /// Sets the field: `four`. Returns the struct itself.
    #[must_use]
    pub fn with_four(mut self, four: Option<Vec<i64>>) -> Self {
        self.set_four(four);
        self
    }
}

impl SignedIntFields {
    //! Field: `five`

    /// Gets the field: `five`.
    #[must_use]
    pub fn five(&self) -> Option<&Vec<i128>> {
        self.five.as_ref()
    }

    /// Sets the field: `five`. Returns the previous value.
    pub fn set_five(&mut self, five: Option<Vec<i128>>) -> Option<Vec<i128>> {
        std::mem::replace(&mut self.five, five)
    }

    /// Sets the field: `five`. Returns the struct itself.
    #[must_use]
    pub fn with_five(mut self, five: Option<Vec<i128>>) -> Self {
        self.set_five(five);
        self
    }
}
