/// A variant with special slice cases.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum SpecialCases {
    /// A `uuid` slice case.
    One(Vec<proto_packet::types::Uuid>),

    /// A `string` slice case.
    Two(Vec<String>),
}
