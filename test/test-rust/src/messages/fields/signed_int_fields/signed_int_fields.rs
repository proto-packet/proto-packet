/// ```pps
/// // A message with signed integer fields.
/// message SignedIntFields {
///    
///     // An `i8` field.
///     one: i8 = 1;
///    
///     // An `i16` field.
///     two: i16 = 2;
///    
///     // An `i32` field.
///     three: i32 = 3;
///    
///     // An `i64` field.
///     four: i64 = 4;
///    
///     // An `i128` field.
///     five: i128 = 5;
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
    one: Option<i8>,
    two: Option<i16>,
    three: Option<i32>,
    four: Option<i64>,
    five: Option<i128>,
}

impl SignedIntFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> Option<i8> {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one<F>(&mut self, one: F) -> Option<i8>
    where
        F: Into<Option<i8>>,
    {
        let one: Option<i8> = one.into();
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one<F>(mut self, one: F) -> Self
    where
        F: Into<Option<i8>>,
    {
        self.set_one(one);
        self
    }
}

impl SignedIntFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> Option<i16> {
        self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two<F>(&mut self, two: F) -> Option<i16>
    where
        F: Into<Option<i16>>,
    {
        let two: Option<i16> = two.into();
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two<F>(mut self, two: F) -> Self
    where
        F: Into<Option<i16>>,
    {
        self.set_two(two);
        self
    }
}

impl SignedIntFields {
    //! Field: `three`

    /// Gets the field: `three`.
    #[must_use]
    pub fn three(&self) -> Option<i32> {
        self.three
    }

    /// Sets the field: `three`. Returns the previous value.
    pub fn set_three<F>(&mut self, three: F) -> Option<i32>
    where
        F: Into<Option<i32>>,
    {
        let three: Option<i32> = three.into();
        std::mem::replace(&mut self.three, three)
    }

    /// Sets the field: `three`. Returns the struct itself.
    #[must_use]
    pub fn with_three<F>(mut self, three: F) -> Self
    where
        F: Into<Option<i32>>,
    {
        self.set_three(three);
        self
    }
}

impl SignedIntFields {
    //! Field: `four`

    /// Gets the field: `four`.
    #[must_use]
    pub fn four(&self) -> Option<i64> {
        self.four
    }

    /// Sets the field: `four`. Returns the previous value.
    pub fn set_four<F>(&mut self, four: F) -> Option<i64>
    where
        F: Into<Option<i64>>,
    {
        let four: Option<i64> = four.into();
        std::mem::replace(&mut self.four, four)
    }

    /// Sets the field: `four`. Returns the struct itself.
    #[must_use]
    pub fn with_four<F>(mut self, four: F) -> Self
    where
        F: Into<Option<i64>>,
    {
        self.set_four(four);
        self
    }
}

impl SignedIntFields {
    //! Field: `five`

    /// Gets the field: `five`.
    #[must_use]
    pub fn five(&self) -> Option<i128> {
        self.five
    }

    /// Sets the field: `five`. Returns the previous value.
    pub fn set_five<F>(&mut self, five: F) -> Option<i128>
    where
        F: Into<Option<i128>>,
    {
        let five: Option<i128> = five.into();
        std::mem::replace(&mut self.five, five)
    }

    /// Sets the field: `five`. Returns the struct itself.
    #[must_use]
    pub fn with_five<F>(mut self, five: F) -> Self
    where
        F: Into<Option<i128>>,
    {
        self.set_five(five);
        self
    }
}
