use crate::common::impl_with_comments;
use crate::{CaseName, CaseNameRef, TypeTag, WithCaseName, WithTypeTag};
use proto_packet::io::{TagNumber, WithTagNumber};

/// A variant case.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VariantCase {
    comments: Vec<String>,
    case_name: CaseName,
    type_tag: TypeTag,
    tag_number: TagNumber,
    is_boxed: bool,
}

impl_with_comments!(VariantCase);

impl VariantCase {
    //! Construction

    /// Creates a new variant case.
    pub fn new<N, T>(case_name: N, type_tag: T, tag_number: TagNumber) -> Self
    where
        N: Into<CaseName>,
        T: Into<TypeTag>,
    {
        Self {
            comments: Vec::default(),
            case_name: case_name.into(),
            type_tag: type_tag.into(),
            tag_number,
            is_boxed: false,
        }
    }
}

impl VariantCase {
    //! Boxing

    /// Checks if the case is boxed. (it breaks a loop of types)
    pub fn is_boxed(&self) -> bool {
        self.is_boxed
    }

    /// Sets if the case is boxed.
    pub fn set_boxed(&mut self, is_boxed: bool) {
        self.is_boxed = is_boxed;
    }

    /// Sets if the case is boxed.
    #[must_use]
    pub fn with_boxed(mut self, is_boxed: bool) -> Self {
        self.set_boxed(is_boxed);
        self
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
