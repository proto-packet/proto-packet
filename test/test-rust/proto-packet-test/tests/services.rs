mod common;

use common::*;
use proto_packet::axum::body::Body;
use proto_packet::axum::http::{Request, StatusCode};
use proto_packet::service::{ServiceError, ServiceErrorReason};
use proto_packet::types::{Date, Uuid};
use proto_packet_test::services::calls::{GreetRequest, GreetResponse, Greeter, greeter_router};
use proto_packet_test::services::empty::{Empty, empty_router};
use proto_packet_test::services::types::{Echo, echo_router};
use proto_packet_test::types::{Color, Label, Point, Shape};
use std::sync::Arc;

/// A greeter that rejects empty names.
struct TestGreeter;

impl TestGreeter {
    /// Gets the name of the `request`, or a bad request error if it has none.
    fn name(request: &GreetRequest) -> Result<&str, ServiceError> {
        request
            .name()
            .filter(|name| !name.is_empty())
            .ok_or(ServiceErrorReason::BadRequest.into())
    }
}

impl Greeter for TestGreeter {
    async fn greet(&self, request: GreetRequest) -> Result<GreetResponse, ServiceError> {
        let name: &str = Self::name(&request)?;
        Ok(GreetResponse::default().with_greeting(format!("Hello, {}!", name)))
    }

    async fn farewell(&self, request: GreetRequest) -> Result<GreetResponse, ServiceError> {
        let name: &str = Self::name(&request)?;
        Ok(GreetResponse::default().with_greeting(format!("Goodbye, {}!", name)))
    }
}

#[tokio::test]
async fn calls_are_routed_by_name() {
    let test_cases: &[(&str, &str)] = &[
        ("greet", r#"{"greeting":"Hello, World!"}"#),
        ("farewell", r#"{"greeting":"Goodbye, World!"}"#),
    ];
    for (call, expected) in test_cases {
        let path: String = format!("/services.calls.Greeter/{}", call);
        let result: (StatusCode, String) = post(
            greeter_router(Arc::new(TestGreeter)),
            &path,
            r#"{"name":"World"}"#,
        )
        .await;
        assert_eq!(result, (StatusCode::OK, expected.to_string()));
    }
}

#[tokio::test]
async fn call_errors_are_responses() {
    let result: (StatusCode, String) = post(
        greeter_router(Arc::new(TestGreeter)),
        "/services.calls.Greeter/greet",
        "{}",
    )
    .await;
    let expected: (StatusCode, String) = (
        StatusCode::BAD_REQUEST,
        r#"{"reason":"BadRequest"}"#.to_string(),
    );
    assert_eq!(result, expected);
}

#[tokio::test]
async fn unknown_calls_are_not_found() {
    let (result, _): (StatusCode, String) = post(
        greeter_router(Arc::new(TestGreeter)),
        "/services.calls.Greeter/unknown",
        r#"{"name":"World"}"#,
    )
    .await;
    assert_eq!(result, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn calls_are_posts() {
    let request: Request<Body> = Request::get("/services.calls.Greeter/greet")
        .body(Body::empty())
        .unwrap();
    let (result, _): (StatusCode, String) =
        send(greeter_router(Arc::new(TestGreeter)), request).await;
    assert_eq!(result, StatusCode::METHOD_NOT_ALLOWED);
}

/// An echo that returns each request.
struct TestEcho;

impl Echo for TestEcho {
    async fn boolean(&self, request: bool) -> Result<bool, ServiceError> {
        Ok(request)
    }

    async fn integer(&self, request: u64) -> Result<u64, ServiceError> {
        Ok(request)
    }

    async fn float(&self, request: f64) -> Result<f64, ServiceError> {
        Ok(request)
    }

    async fn id(&self, request: Uuid) -> Result<Uuid, ServiceError> {
        Ok(request)
    }

    async fn text(&self, request: String) -> Result<String, ServiceError> {
        Ok(request)
    }

    async fn time(&self, request: i64) -> Result<i64, ServiceError> {
        Ok(request)
    }

    async fn day(&self, request: Date) -> Result<Date, ServiceError> {
        Ok(request)
    }

    async fn bytes(&self, request: Vec<u8>) -> Result<Vec<u8>, ServiceError> {
        Ok(request)
    }

    async fn point(&self, request: Point) -> Result<Point, ServiceError> {
        Ok(request)
    }

    async fn label(&self, request: Label) -> Result<Label, ServiceError> {
        Ok(request)
    }

    async fn color(&self, request: Color) -> Result<Color, ServiceError> {
        Ok(request)
    }

    async fn shape(&self, request: Shape) -> Result<Shape, ServiceError> {
        Ok(request)
    }
}

#[tokio::test]
async fn calls_of_each_kind_of_type() {
    let max: String = u64::MAX.to_string();
    let test_cases: &[(&str, &str)] = &[
        ("boolean", "true"),
        ("integer", &max),
        ("float", "-2.5"),
        ("id", ID_JSON),
        ("text", r#""text""#),
        ("time", "-1"),
        ("day", DAY_JSON),
        ("bytes", "[1,2]"),
        ("point", POINT_JSON),
        ("label", LABEL_JSON),
        ("color", r#""Green""#),
        ("shape", SHAPE_JSON),
    ];
    for (call, json) in test_cases {
        let path: String = format!("/services.types.Echo/{}", call);
        let result: (StatusCode, String) = post(echo_router(Arc::new(TestEcho)), &path, json).await;
        assert_eq!(result, (StatusCode::OK, json.to_string()), "call: {}", call);
    }
}

/// A service without calls.
struct TestEmpty;

impl Empty for TestEmpty {}

#[tokio::test]
async fn services_without_calls_route_nothing() {
    let (result, _): (StatusCode, String) = post(
        empty_router(Arc::new(TestEmpty)),
        "/services.empty.Empty/call",
        "{}",
    )
    .await;
    assert_eq!(result, StatusCode::NOT_FOUND);
}
