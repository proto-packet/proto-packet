/// ```pps
/// // A message with unsigned integer fields.
/// message UnsignedIntFields {
///    
///     // A `u8` field.
///     one: u8 = 1;
///    
///     // A `u16` field.
///     two: u16 = 2;
///    
///     // A `u32` field.
///     three: u32 = 3;
///    
///     // A `u64` field.
///     four: u64 = 4;
///    
///     // A `u128` field.
///     five: u128 = 5;
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
    one: Option<u8>,
    two: Option<u16>,
    three: Option<u32>,
    four: Option<u64>,
    five: Option<u128>,
}

impl UnsignedIntFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<u8> {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: Option<u8>) -> Option<u8> {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: Option<u8>) -> Self {
        self.set_one(one);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<u16> {
        self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: Option<u16>) -> Option<u16> {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: Option<u16>) -> Self {
        self.set_two(two);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `three`

    /// Gets the field: `three`.
    #[must_use]
    pub fn three(&self) -> Option<u32> {
        self.three
    }

    /// Sets the field: `three`. Returns the previous value.
    pub fn set_three(&mut self, three: Option<u32>) -> Option<u32> {
        std::mem::replace(&mut self.three, three)
    }

    /// Sets the field: `three`. Returns the struct itself.
    #[must_use]
    pub fn with_three(mut self, three: Option<u32>) -> Self {
        self.set_three(three);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `four`

    /// Gets the field: `four`.
    #[must_use]
    pub fn four(&self) -> Option<u64> {
        self.four
    }

    /// Sets the field: `four`. Returns the previous value.
    pub fn set_four(&mut self, four: Option<u64>) -> Option<u64> {
        std::mem::replace(&mut self.four, four)
    }

    /// Sets the field: `four`. Returns the struct itself.
    #[must_use]
    pub fn with_four(mut self, four: Option<u64>) -> Self {
        self.set_four(four);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `five`

    /// Gets the field: `five`.
    #[must_use]
    pub fn five(&self) -> Option<u128> {
        self.five
    }

    /// Sets the field: `five`. Returns the previous value.
    pub fn set_five(&mut self, five: Option<u128>) -> Option<u128> {
        std::mem::replace(&mut self.five, five)
    }

    /// Sets the field: `five`. Returns the struct itself.
    #[must_use]
    pub fn with_five(mut self, five: Option<u128>) -> Self {
        self.set_five(five);
        self
    }
}
