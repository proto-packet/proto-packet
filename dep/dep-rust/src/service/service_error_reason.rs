use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// The reason a [crate::service::ServiceError] was raised.
///
/// Variants map to common HTTP status codes used in API responses.
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
    #[must_use]
    pub const fn status_code(self) -> u16 {
        match self {
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::MethodNotAllowed => 405,
            Self::RequestTimeout => 408,
            Self::Conflict => 409,
            Self::TooManyRequests => 429,
            Self::InternalServerError => 500,
            Self::NotImplemented => 501,
            Self::BadGateway => 502,
            Self::ServiceUnavailable => 503,
            Self::GatewayTimeout => 504,
        }
    }

    /// Gets the standard reason phrase.
    #[must_use]
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
        write!(f, "{} {}", self.status_code(), self.reason_phrase())
    }
}
