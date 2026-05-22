/// A variant with a single boolean case.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum BooleanCase {
    /// A `bool` case.
    One(bool),
}
