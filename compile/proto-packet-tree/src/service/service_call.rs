use crate::common::impl_with_comments;
use crate::{CallName, CallNameRef, TypeTag, WithCallName};

/// A service call.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ServiceCall {
    comments: Vec<String>,
    call_name: CallName,
    request: TypeTag,
    response: TypeTag,
}

impl_with_comments!(ServiceCall);

impl ServiceCall {
    //! Construction

    /// Creates a new service call.
    pub fn new<N, Q, R>(call_name: N, request: Q, response: R) -> Self
    where
        N: Into<CallName>,
        Q: Into<TypeTag>,
        R: Into<TypeTag>,
    {
        Self {
            comments: Vec::default(),
            call_name: call_name.into(),
            request: request.into(),
            response: response.into(),
        }
    }
}

impl ServiceCall {
    //! Properties

    /// Gets the request type.
    pub fn request(&self) -> &TypeTag {
        &self.request
    }

    /// Gets the response type.
    pub fn response(&self) -> &TypeTag {
        &self.response
    }
}

impl WithCallName for ServiceCall {
    fn call_name(&self) -> CallNameRef<'_> {
        self.call_name.to_ref()
    }
}
