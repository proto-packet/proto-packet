/// ```pps
/// // A struct with unsigned integer slice fields.
/// struct UnsignedIntFields {
///    
///     // A `u8` slice field.
///     one: []u8;
///    
///     // A `u16` slice field.
///     two: []u16;
///    
///     // A `u32` slice field.
///     three: []u32;
///    
///     // A `u64` slice field.
///     four: []u64;
///    
///     // A `u128` slice field.
///     five: []u128;
/// }
/// ```
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct UnsignedIntFields {
    one: Vec<u8>,
    two: Vec<u16>,
    three: Vec<u32>,
    four: Vec<u64>,
    five: Vec<u128>,
}

impl UnsignedIntFields {
    //! Construction

    /// Creates a new [UnsignedIntFields].
    pub const fn new(
        one: Vec<u8>,
        two: Vec<u16>,
        three: Vec<u32>,
        four: Vec<u64>,
        five: Vec<u128>,
    ) -> Self {
        Self {
            one,
            two,
            three,
            four,
            five,
        }
    }

    /// Creates a new [UnsignedIntFields].
    pub fn from<F1, F2, F3, F4, F5>(one: F1, two: F2, three: F3, four: F4, five: F5) -> Self
    where
        F1: Into<Vec<u8>>,
        F2: Into<Vec<u16>>,
        F3: Into<Vec<u32>>,
        F4: Into<Vec<u64>>,
        F5: Into<Vec<u128>>,
    {
        Self::new(
            one.into(),
            two.into(),
            three.into(),
            four.into(),
            five.into(),
        )
    }
}

impl UnsignedIntFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> &Vec<u8> {
        &self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Vec<u8>) -> Vec<u8> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Vec<u8>) -> Self {
        self.set_one(one);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> &Vec<u16> {
        &self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Vec<u16>) -> Vec<u16> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Vec<u16>) -> Self {
        self.set_two(two);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `three`

    /// Gets the field: `three`.
    #[must_use]
    pub fn three(&self) -> &Vec<u32> {
        &self.three
    }

    /// Sets the field: `three`. Returns the previous value.
    pub fn set_three(&mut self, three: Vec<u32>) -> Vec<u32> {
        std::mem::replace(&mut self.three, three)
    }

    /// Sets the field: `three`. Returns the struct itself.
    #[must_use]
    pub fn with_three(mut self, three: Vec<u32>) -> Self {
        self.set_three(three);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `four`

    /// Gets the field: `four`.
    #[must_use]
    pub fn four(&self) -> &Vec<u64> {
        &self.four
    }

    /// Sets the field: `four`. Returns the previous value.
    pub fn set_four(&mut self, four: Vec<u64>) -> Vec<u64> {
        std::mem::replace(&mut self.four, four)
    }

    /// Sets the field: `four`. Returns the struct itself.
    #[must_use]
    pub fn with_four(mut self, four: Vec<u64>) -> Self {
        self.set_four(four);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `five`

    /// Gets the field: `five`.
    #[must_use]
    pub fn five(&self) -> &Vec<u128> {
        &self.five
    }

    /// Sets the field: `five`. Returns the previous value.
    pub fn set_five(&mut self, five: Vec<u128>) -> Vec<u128> {
        std::mem::replace(&mut self.five, five)
    }

    /// Sets the field: `five`. Returns the struct itself.
    #[must_use]
    pub fn with_five(mut self, five: Vec<u128>) -> Self {
        self.set_five(five);
        self
    }
}
