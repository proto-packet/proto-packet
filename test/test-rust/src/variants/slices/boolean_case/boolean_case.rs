/// A variant with a single boolean slice case.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum BooleanCase {
    /// A `bool` slice case.
    One(Vec<bool>),
}
