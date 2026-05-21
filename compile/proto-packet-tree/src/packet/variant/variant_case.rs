use crate::{CaseName, CaseNameRef, TypeTag, WithCaseName, WithTypeTag};
use proto_packet::io::{TagNumber, WithTagNumber};

/// A variant case.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VariantCase {
    pub(crate) comments: Vec<String>,
    case_name: CaseName,
    type_tag: TypeTag,
    tag_number: TagNumber,
}

impl VariantCase {
    //! Construction

    /// Creates a new variant case.
    pub fn new<N, T>(case_name: N, type_tag: T, tag_number: TagNumber) -> Self
    where
        N: Into<CaseName>,
        T: Into<TypeTag>,
    {
        let case_name: CaseName = case_name.into();
        let type_tag: TypeTag = type_tag.into();
        Self {
            comments: Vec::default(),
            case_name,
            type_tag,
            tag_number,
        }
    }
}

impl WithCaseName for VariantCase {
    fn case_name(&self) -> CaseNameRef<'_> {
        self.case_name.to_ref()
    }
}

impl WithTypeTag for VariantCase {
    fn type_tag(&self) -> &TypeTag {
        &self.type_tag
    }
}

impl WithTagNumber for VariantCase {
    fn tag_number(&self) -> TagNumber {
        self.tag_number
    }
}
