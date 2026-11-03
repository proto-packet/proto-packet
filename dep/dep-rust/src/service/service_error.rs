use crate::service::ServiceErrorReason;
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// An error returned by a service call.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ServiceError {
    reason: ServiceErrorReason,
}

impl ServiceError {
    //! Construction

    /// Creates a new service error with the `reason`.
    pub const fn new(reason: ServiceErrorReason) -> Self {
        Self { reason }
    }
}

impl From<ServiceErrorReason> for ServiceError {
    fn from(reason: ServiceErrorReason) -> Self {
        Self::new(reason)
    }
}

impl From<JsonRejection> for ServiceError {
    /// Converts a request body that isn't valid JSON for a call. (a missing JSON content type is an
    /// `UnsupportedMediaType`, anything else a `BadRequest`)
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::MissingJsonContentType(_) => {
                Self::new(ServiceErrorReason::UnsupportedMediaType)
            }
            _ => Self::new(ServiceErrorReason::BadRequest),
        }
    }
}

impl ServiceError {
    //! Properties

    /// Gets the reason.
    pub const fn reason(&self) -> ServiceErrorReason {
        self.reason
    }
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.reason, f)
    }
}

impl std::error::Error for ServiceError {}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        (self.reason.status_code(), Json(self)).into_response()
    }
}
