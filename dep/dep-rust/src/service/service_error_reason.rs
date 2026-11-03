use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// The reason for a service error. (each is an HTTP status)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ServiceErrorReason {
    /// 400 Bad Request.
    BadRequest,

    /// 401 Unauthorized.
    Unauthorized,

    /// 403 Forbidden.
    Forbidden,

    /// 404 Not Found.
    NotFound,

    /// 405 Method Not Allowed.
    MethodNotAllowed,

    /// 408 Request Timeout.
    RequestTimeout,

    /// 409 Conflict.
    Conflict,

    /// 429 Too Many Requests.
    TooManyRequests,

    /// 500 Internal Server Error.
    InternalServerError,

    /// 501 Not Implemented.
    NotImplemented,

    /// 502 Bad Gateway.
    BadGateway,

    /// 503 Service Unavailable.
    ServiceUnavailable,

    /// 504 Gateway Timeout.
    GatewayTimeout,
}

impl ServiceErrorReason {
    //! Properties

    /// Gets the HTTP status code.
    pub const fn status_code(self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
            Self::Conflict => StatusCode::CONFLICT,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            Self::BadGateway => StatusCode::BAD_GATEWAY,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }

    /// Gets the HTTP reason phrase. (ex: `Not Found`)
    pub const fn reason_phrase(self) -> &'static str {
        match self {
            Self::BadRequest => "Bad Request",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::RequestTimeout => "Request Timeout",
            Self::Conflict => "Conflict",
            Self::TooManyRequests => "Too Many Requests",
            Self::InternalServerError => "Internal Server Error",
            Self::NotImplemented => "Not Implemented",
            Self::BadGateway => "Bad Gateway",
            Self::ServiceUnavailable => "Service Unavailable",
            Self::GatewayTimeout => "Gateway Timeout",
        }
    }
}

impl Display for ServiceErrorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.status_code().as_u16(),
            self.reason_phrase()
        )
    }
}
