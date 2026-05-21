use crate::{CaseName, CaseNameRef, WithCaseName};
use proto_packet::io::{TagNumber, WithTagNumber};

/// An enum case.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct EnumCase {
    pub(crate) comments: Vec<String>,
    case_name: CaseName,
    tag_number: TagNumber,
}

impl EnumCase {
    //! Construction

    /// Creates a new enum case.
    pub fn new<N>(case_name: N, tag_number: TagNumber) -> Self
    where
        N: Into<CaseName>,
    {
        let case_name: CaseName = case_name.into();
        Self {
            comments: Vec::default(),
            case_name,
            tag_number,
        }
    }
}

impl WithCaseName for EnumCase {
    fn case_name(&self) -> CaseNameRef<'_> {
        self.case_name.to_ref()
    }
}

impl WithTagNumber for EnumCase {
    fn tag_number(&self) -> TagNumber {
        self.tag_number
    }
}
