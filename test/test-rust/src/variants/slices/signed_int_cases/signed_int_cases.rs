/// A variant with signed integer slice cases.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum SignedIntCases {
    /// An `i8` slice case.
    One(Vec<i8>),

    /// An `i16` slice case.
    Two(Vec<i16>),

    /// An `i32` slice case.
    Three(Vec<i32>),

    /// An `i64` slice case.
    Four(Vec<i64>),

    /// An `i128` slice case.
    Five(Vec<i128>),
}
