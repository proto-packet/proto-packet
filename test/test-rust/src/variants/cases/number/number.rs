/// A numeric value of varying precision.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Number {
    Small(u8),
    
    Medium(u32),
    
    Large(u128),
}
