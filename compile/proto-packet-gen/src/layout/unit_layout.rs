/// A unit of a struct's fixed block.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum UnitLayout {
    /// The fixed-size field at the `index`, in declaration order.
    Field { index: usize },

    /// The end table.
    EndTable,

    /// The bit group, at the `offset` & of the `size` in bytes.
    Bits { offset: usize, size: usize },

    /// The padding, of the `size` in bytes.
    Padding { size: usize },
}
