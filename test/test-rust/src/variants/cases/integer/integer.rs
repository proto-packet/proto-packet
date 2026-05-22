/// A signed or unsigned integer.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Integer {
    /// A signed 64-bit integer.
    Signed(i64),
    
    /// An unsigned 64-bit integer.
    Unsigned(u64),
}
