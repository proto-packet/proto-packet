/// A variant with special cases.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum SpecialCases {
    /// A `uuid` case.
    One(proto_packet::types::Uuid),

    /// A `string` case.
    Two(String),
}
