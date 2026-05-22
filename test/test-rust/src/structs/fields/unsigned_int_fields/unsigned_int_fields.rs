/// ```pps
/// // A struct with unsigned integer fields.
/// struct UnsignedIntFields {
///    
///     // A `u8` field.
///     one: u8;
///    
///     // A `u16` field.
///     two: u16;
///    
///     // A `u32` field.
///     three: u32;
///    
///     // A `u64` field.
///     four: u64;
///    
///     // A `u128` field.
///     five: u128;
/// }
/// ```
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct UnsignedIntFields {
    one: u8,
    two: u16,
    three: u32,
    four: u64,
    five: u128,
}

impl UnsignedIntFields {
    //! Construction

    /// Creates a new [UnsignedIntFields].
    pub const fn new(one: u8, two: u16, three: u32, four: u64, five: u128) -> Self {
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
        F1: Into<u8>,
        F2: Into<u16>,
        F3: Into<u32>,
        F4: Into<u64>,
        F5: Into<u128>,
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
    pub fn one(&self) -> u8 {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: u8) -> u8 {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: u8) -> Self {
        self.set_one(one);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> u16 {
        self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: u16) -> u16 {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: u16) -> Self {
        self.set_two(two);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `three`

    /// Gets the field: `three`.
    #[must_use]
    pub fn three(&self) -> u32 {
        self.three
    }

    /// Sets the field: `three`. Returns the previous value.
    pub fn set_three(&mut self, three: u32) -> u32 {
        std::mem::replace(&mut self.three, three)
    }

    /// Sets the field: `three`. Returns the struct itself.
    #[must_use]
    pub fn with_three(mut self, three: u32) -> Self {
        self.set_three(three);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `four`

    /// Gets the field: `four`.
    #[must_use]
    pub fn four(&self) -> u64 {
        self.four
    }

    /// Sets the field: `four`. Returns the previous value.
    pub fn set_four(&mut self, four: u64) -> u64 {
        std::mem::replace(&mut self.four, four)
    }

    /// Sets the field: `four`. Returns the struct itself.
    #[must_use]
    pub fn with_four(mut self, four: u64) -> Self {
        self.set_four(four);
        self
    }
}

impl UnsignedIntFields {
    //! Field: `five`

    /// Gets the field: `five`.
    #[must_use]
    pub fn five(&self) -> u128 {
        self.five
    }

    /// Sets the field: `five`. Returns the previous value.
    pub fn set_five(&mut self, five: u128) -> u128 {
        std::mem::replace(&mut self.five, five)
    }

    /// Sets the field: `five`. Returns the struct itself.
    #[must_use]
    pub fn with_five(mut self, five: u128) -> Self {
        self.set_five(five);
        self
    }
}
