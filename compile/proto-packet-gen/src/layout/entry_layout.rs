/// An entry of a struct's tail.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EntryLayout {
    /// The index of the entry's field, in declaration order.
    pub field: usize,

    /// The alignment of the entry.
    pub alignment: usize,
}
