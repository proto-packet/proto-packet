use crate::service::ServiceErrorReason;
use axum::Json;
use axum::http::StatusCode;
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

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status: StatusCode = StatusCode::from_u16(self.reason.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}
