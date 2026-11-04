use crate::layout::{BitLayout, ValueLayout};

/// The layout of a struct field.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FieldLayout {
    /// The place of the value.
    pub value: ValueLayout,

    /// The presence bit of an optional field. (an optional enum has none, since it uses tag 0)
    pub presence: Option<BitLayout>,
}
