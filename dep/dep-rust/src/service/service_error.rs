use crate::service::ServiceErrorReason;
use std::fmt::{Display, Formatter};

/// An error returned by a service call.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ServiceError {
    reason: ServiceErrorReason,
}

impl ServiceError {
    //! Construction

    /// Creates a new [ServiceError] with the `reason`.
    #[must_use]
    pub const fn new(reason: ServiceErrorReason) -> Self {
        Self { reason }
    }
}

impl ServiceError {
    //! Properties

    /// Gets the reason.
    #[must_use]
    pub fn reason(&self) -> ServiceErrorReason {
        self.reason
    }
}

impl From<ServiceErrorReason> for ServiceError {
    fn from(reason: ServiceErrorReason) -> Self {
        Self::new(reason)
    }
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.reason, f)
    }
}

impl std::error::Error for ServiceError {}
