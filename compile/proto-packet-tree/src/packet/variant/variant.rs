use crate::{TreeError, TypeName, TypeNameRef, VariantCase, WithCaseName, WithTypeName};
use proto_packet::io::{TagNumber, WithTagNumber};

/// A variant.
///
/// # Invariants
/// 1. No two cases can have the same name.
/// 2. No two cases can have the same tag number.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Variant {
    pub(crate) comments: Vec<String>,
    type_name: TypeName,
    cases: Vec<VariantCase>,
}

impl From<TypeName> for Variant {
    fn from(type_name: TypeName) -> Self {
        Self {
            comments: Vec::default(),
            type_name,
            cases: Vec::default(),
        }
    }
}

impl Variant {
    //! Cases

    /// Gets the cases.
    pub fn cases(&self) -> &[VariantCase] {
        self.cases.as_slice()
    }

    /// Gets the optional case with the `case_name`.
    pub fn case_with_name<S>(&self, case_name: S) -> Option<&VariantCase>
    where
        S: AsRef<str>,
    {
        self.cases.iter().find(|case| case.case_name() == case_name)
    }

    /// Gets the optional case with the `tag_number`.
    pub fn case_with_tag_number(&self, tag_number: TagNumber) -> Option<&VariantCase> {
        self.cases
            .iter()
            .find(|case| case.tag_number() == tag_number)
    }

    /// Adds the `case`.
    pub fn add_case<C>(&mut self, case: C) -> Result<(), TreeError>
    where
        C: Into<VariantCase>,
    {
        let case: VariantCase = case.into();

        if self.case_with_name(case.case_name()).is_some() {
            return Err(TreeError::DuplicateCaseName {
                type_name: self.type_name.clone(),
                case_name: case.case_name().into_owned(),
            });
        }
        if self.case_with_tag_number(case.tag_number()).is_some() {
            return Err(TreeError::DuplicateTagNumber {
                type_name: self.type_name.clone(),
                tag_number: case.tag_number(),
            });
        }

        self.cases.push(case);

        Ok(())
    }

    /// Adds the `case`.
    pub fn with_case<C>(mut self, case: C) -> Result<Self, TreeError>
    where
        C: Into<VariantCase>,
    {
        self.add_case(case)?;
        Ok(self)
    }
}

impl WithTypeName for Variant {
    fn type_name(&self) -> TypeNameRef<'_> {
        self.type_name.to_ref()
    }
}
