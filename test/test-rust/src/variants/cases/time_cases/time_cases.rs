/// A variant with time cases.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum TimeCases {
    /// A `timestamp` case.
    One(i64),
}
