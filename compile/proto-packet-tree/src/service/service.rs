use crate::{ServiceCall, TreeError, TypeName, TypeNameRef, WithCallName, WithTypeName};

/// A service.
///
/// # Invariants
/// 1. No two calls can have the same name.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Service {
    pub(crate) comments: Vec<String>,
    type_name: TypeName,
    calls: Vec<ServiceCall>,
}

impl From<TypeName> for Service {
    fn from(type_name: TypeName) -> Self {
        Self {
            comments: Vec::default(),
            type_name,
            calls: Vec::default(),
        }
    }
}

impl Service {
    //! Calls

    /// Gets the calls.
    pub fn calls(&self) -> &[ServiceCall] {
        self.calls.as_slice()
    }

    /// Gets the optional call with the `call_name`.
    pub fn call_with_name<S>(&self, call_name: S) -> Option<&ServiceCall>
    where
        S: AsRef<str>,
    {
        self.calls.iter().find(|call| call.call_name() == call_name)
    }

    /// Adds the `call`.
    pub fn add_call<C>(&mut self, call: C) -> Result<(), TreeError>
    where
        C: Into<ServiceCall>,
    {
        let call: ServiceCall = call.into();

        if self.call_with_name(call.call_name()).is_some() {
            return Err(TreeError::DuplicateCallName {
                type_name: self.type_name.clone(),
                call_name: call.call_name().into_owned(),
            });
        }

        self.calls.push(call);

        Ok(())
    }

    /// Adds the `call`.
    pub fn with_call<C>(mut self, call: C) -> Result<Self, TreeError>
    where
        C: Into<ServiceCall>,
    {
        self.add_call(call)?;
        Ok(self)
    }
}

impl WithTypeName for Service {
    fn type_name(&self) -> TypeNameRef<'_> {
        self.type_name.to_ref()
    }
}
