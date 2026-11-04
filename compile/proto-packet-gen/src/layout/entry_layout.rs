/// An entry of a struct's tail.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EntryLayout {
    /// The index of the entry's field, in declaration order.
    pub field: usize,

    /// The alignment of the entry.
    pub alignment: usize,

    /// The alignment to pad the entry to. (the next entry's alignment, 1 for the last entry)
    pub padded_alignment: usize,
}
