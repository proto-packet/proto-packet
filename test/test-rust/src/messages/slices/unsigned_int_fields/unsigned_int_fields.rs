/// ```pps
/// // A message with unsigned integer slice fields.
/// message UnsignedIntFields {
///    
///     // A `u8` slice field.
///     one: []u8 = 1;
///    
///     // A `u16` slice field.
///     two: []u16 = 2;
///    
///     // A `u32` slice field.
///     three: []u32 = 3;
///    
///     // A `u64` slice field.
///     four: []u64 = 4;
///    
///     // A `u128` slice field.
///     five: []u128 = 5;
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
    serde::Serialize,
    serde::Deserialize,
)]
pub struct UnsignedIntFields {
    one: Option<Vec<u8>>,
    two: Option<Vec<u16>>,
    three: Option<Vec<u32>>,
    four: Option<Vec<u64>>,
    five: Option<Vec<u128>>,
}

impl UnsignedIntFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<&Vec<u8>> {
        self.one.as_ref()
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<Vec<u8>>) -> Option<Vec<u8>> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<Vec<u8>>) -> Self {
        self.set_one(one);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<&Vec<u16>> {
        self.two.as_ref()
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Option<Vec<u16>>) -> Option<Vec<u16>> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Option<Vec<u16>>) -> Self {
        self.set_two(two);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `three`

    /// Gets the field: `three`.
    #[must_use]
    pub fn three(&self) -> Option<&Vec<u32>> {
        self.three.as_ref()
    }

    /// Sets the field: `three`. Returns the previous value.
    pub fn set_three(&mut self, three: Option<Vec<u32>>) -> Option<Vec<u32>> {
        std::mem::replace(&mut self.three, three)
    }

    /// Sets the field: `three`. Returns the struct itself.
    #[must_use]
    pub fn with_three(mut self, three: Option<Vec<u32>>) -> Self {
        self.set_three(three);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `four`

    /// Gets the field: `four`.
    #[must_use]
    pub fn four(&self) -> Option<&Vec<u64>> {
        self.four.as_ref()
    }

    /// Sets the field: `four`. Returns the previous value.
    pub fn set_four(&mut self, four: Option<Vec<u64>>) -> Option<Vec<u64>> {
        std::mem::replace(&mut self.four, four)
    }

    /// Sets the field: `four`. Returns the struct itself.
    #[must_use]
    pub fn with_four(mut self, four: Option<Vec<u64>>) -> Self {
        self.set_four(four);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `five`

    /// Gets the field: `five`.
    #[must_use]
    pub fn five(&self) -> Option<&Vec<u128>> {
        self.five.as_ref()
    }

    /// Sets the field: `five`. Returns the previous value.
    pub fn set_five(&mut self, five: Option<Vec<u128>>) -> Option<Vec<u128>> {
        std::mem::replace(&mut self.five, five)
    }

    /// Sets the field: `five`. Returns the struct itself.
    #[must_use]
    pub fn with_five(mut self, five: Option<Vec<u128>>) -> Self {
        self.set_five(five);
        self
    }
}
