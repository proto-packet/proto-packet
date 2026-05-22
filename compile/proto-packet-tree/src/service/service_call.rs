use crate::{CallName, CallNameRef, TypeTag, WithCallName};

/// A service call.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ServiceCall {
    pub(crate) comments: Vec<String>,
    call_name: CallName,
    request: TypeTag,
    response: TypeTag,
}

impl ServiceCall {
    //! Construction

    /// Creates a new service call.
    pub fn new<N, Q, S>(call_name: N, request: Q, response: S) -> Self
    where
        N: Into<CallName>,
        Q: Into<TypeTag>,
        S: Into<TypeTag>,
    {
        let call_name: CallName = call_name.into();
        let request: TypeTag = request.into();
        let response: TypeTag = response.into();
        Self {
            comments: Vec::default(),
            call_name,
            request,
            response,
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
