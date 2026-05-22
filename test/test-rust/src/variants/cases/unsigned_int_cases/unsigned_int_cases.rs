/// A variant with unsigned integer cases.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum UnsignedIntCases {
    /// A `u8` case.
    One(u8),
    
    /// A `u16` case.
    Two(u16),
    
    /// A `u32` case.
    Three(u32),
    
    /// A `u64` case.
    Four(u64),
    
    /// A `u128` case.
    Five(u128),
}
