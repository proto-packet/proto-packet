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
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Default,
    proto_packet::serde::Serialize,
    proto_packet::serde::Deserialize,
)]
#[serde(crate = "proto_packet::serde")]
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
    pub fn set_one<F>(&mut self, one: F) -> Option<Vec<i8>>
    where
        F: Into<Option<Vec<i8>>>,
    {
        let one: Option<Vec<i8>> = one.into();
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one<F>(mut self, one: F) -> Self
    where
        F: Into<Option<Vec<i8>>>,
    {
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
    pub fn set_two<F>(&mut self, two: F) -> Option<Vec<i16>>
    where
        F: Into<Option<Vec<i16>>>,
    {
        let two: Option<Vec<i16>> = two.into();
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two<F>(mut self, two: F) -> Self
    where
        F: Into<Option<Vec<i16>>>,
    {
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
    pub fn set_three<F>(&mut self, three: F) -> Option<Vec<i32>>
    where
        F: Into<Option<Vec<i32>>>,
    {
        let three: Option<Vec<i32>> = three.into();
        std::mem::replace(&mut self.three, three)
    }

    /// Sets the field: `three`. Returns the struct itself.
    #[must_use]
    pub fn with_three<F>(mut self, three: F) -> Self
    where
        F: Into<Option<Vec<i32>>>,
    {
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
    pub fn set_four<F>(&mut self, four: F) -> Option<Vec<i64>>
    where
        F: Into<Option<Vec<i64>>>,
    {
        let four: Option<Vec<i64>> = four.into();
        std::mem::replace(&mut self.four, four)
    }

    /// Sets the field: `four`. Returns the struct itself.
    #[must_use]
    pub fn with_four<F>(mut self, four: F) -> Self
    where
        F: Into<Option<Vec<i64>>>,
    {
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
    pub fn set_five<F>(&mut self, five: F) -> Option<Vec<i128>>
    where
        F: Into<Option<Vec<i128>>>,
    {
        let five: Option<Vec<i128>> = five.into();
        std::mem::replace(&mut self.five, five)
    }

    /// Sets the field: `five`. Returns the struct itself.
    #[must_use]
    pub fn with_five<F>(mut self, five: F) -> Self
    where
        F: Into<Option<Vec<i128>>>,
    {
        self.set_five(five);
        self
    }
}
