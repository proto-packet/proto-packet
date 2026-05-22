/// A variant with signed integer cases.
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize,
)]
pub enum SignedIntCases {
    /// An `i8` case.
    One(i8),

    /// An `i16` case.
    Two(i16),

    /// An `i32` case.
    Three(i32),

    /// An `i64` case.
    Four(i64),

    /// An `i128` case.
    Five(i128),
}
