/// ```pps
/// // A struct with signed integer slice fields.
/// struct SignedIntFields {
///    
///     // An `i8` slice field.
///     one: []i8;
///    
///     // An `i16` slice field.
///     two: []i16;
///    
///     // An `i32` slice field.
///     three: []i32;
///    
///     // An `i64` slice field.
///     four: []i64;
///    
///     // An `i128` slice field.
///     five: []i128;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct SignedIntFields {
    one: Vec<i8>,
    two: Vec<i16>,
    three: Vec<i32>,
    four: Vec<i64>,
    five: Vec<i128>,
}

impl SignedIntFields {
    //! Construction

    /// Creates a new [SignedIntFields].
    pub const fn new(one: Vec<i8>, two: Vec<i16>, three: Vec<i32>, four: Vec<i64>, five: Vec<i128>) -> Self {
        Self { one, two, three, four, five }
    }

    /// Creates a new [SignedIntFields].
    pub fn from<F1, F2, F3, F4, F5>(one: F1, two: F2, three: F3, four: F4, five: F5) -> Self where F1: Into<Vec<i8>>, F2: Into<Vec<i16>>, F3: Into<Vec<i32>>, F4: Into<Vec<i64>>, F5: Into<Vec<i128>> {
        Self::new(one.into(), two.into(), three.into(), four.into(), five.into())
    }
}

impl SignedIntFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> &Vec<i8> {
        &self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Vec<i8>) -> Vec<i8> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Vec<i8>) -> Self {
        self.set_one(one);
        self
    }
}

impl SignedIntFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> &Vec<i16> {
        &self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Vec<i16>) -> Vec<i16> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Vec<i16>) -> Self {
        self.set_two(two);
        self
    }
}

impl SignedIntFields {
    //! Field: `three`

    /// Gets the field: `three`.
    #[must_use]
    pub fn three(&self) -> &Vec<i32> {
        &self.three
    }

    /// Sets the field: `three`. Returns the previous value.
    pub fn set_three(&mut self, three: Vec<i32>) -> Vec<i32> {
        std::mem::replace(&mut self.three, three)
    }

    /// Sets the field: `three`. Returns the struct itself.
    #[must_use]
    pub fn with_three(mut self, three: Vec<i32>) -> Self {
        self.set_three(three);
        self
    }
}

impl SignedIntFields {
    //! Field: `four`

    /// Gets the field: `four`.
    #[must_use]
    pub fn four(&self) -> &Vec<i64> {
        &self.four
    }

    /// Sets the field: `four`. Returns the previous value.
    pub fn set_four(&mut self, four: Vec<i64>) -> Vec<i64> {
        std::mem::replace(&mut self.four, four)
    }

    /// Sets the field: `four`. Returns the struct itself.
    #[must_use]
    pub fn with_four(mut self, four: Vec<i64>) -> Self {
        self.set_four(four);
        self
    }
}

impl SignedIntFields {
    //! Field: `five`

    /// Gets the field: `five`.
    #[must_use]
    pub fn five(&self) -> &Vec<i128> {
        &self.five
    }

    /// Sets the field: `five`. Returns the previous value.
    pub fn set_five(&mut self, five: Vec<i128>) -> Vec<i128> {
        std::mem::replace(&mut self.five, five)
    }

    /// Sets the field: `five`. Returns the struct itself.
    #[must_use]
    pub fn with_five(mut self, five: Vec<i128>) -> Self {
        self.set_five(five);
        self
    }
}
