use proto_packet::axum::Router;
use proto_packet::axum::body::{Body, to_bytes};
use proto_packet::axum::http::{Request, StatusCode};
use proto_packet::service::{ServiceError, ServiceErrorReason};
use proto_packet_test::names::keywords::{KeywordService, keyword_service_router};
use proto_packet_test::services::calls::{GreetRequest, GreetResponse, Greeter, greeter_router};
use proto_packet_test::types::{Color, Point};
use std::sync::Arc;
use tower::ServiceExt;

/// The test greeter.
struct TestGreeter;

impl Greeter for TestGreeter {
    async fn greet(&self, request: GreetRequest) -> Result<GreetResponse, ServiceError> {
        let name: &str = request.name().ok_or(ServiceErrorReason::BadRequest)?;
        Ok(GreetResponse::default().with_greeting(format!("Hello, {}!", name)))
    }

    async fn step(&self, request: Point) -> Result<Point, ServiceError> {
        Ok(Point::new(request.x() + 1, request.y() + 1))
    }

    async fn color(&self, request: u32) -> Result<Color, ServiceError> {
        Color::from_tag_number_optional(request).ok_or(ServiceErrorReason::NotFound.into())
    }
}

/// The test keyword service.
struct TestKeywordService;

impl KeywordService for TestKeywordService {
    async fn r#type(&self, request: u32) -> Result<u32, ServiceError> {
        Ok(request + 1)
    }

    async fn self_(&self, request: u32) -> Result<u32, ServiceError> {
        Ok(request + 2)
    }
}

/// Posts the JSON `body` to the `path` of the `router`. Returns the response status & body.
async fn post(router: Router, path: &str, body: &str) -> (StatusCode, String) {
    let request: Request<Body> = Request::post(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    send(router, request).await
}

/// Sends the `request` to the `router`. Returns the response status & body.
async fn send(router: Router, request: Request<Body>) -> (StatusCode, String) {
    let response = router.oneshot(request).await.unwrap();
    let status: StatusCode = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

/// Gets the router of the test greeter.
fn greeter() -> Router {
    greeter_router(Arc::new(TestGreeter))
}

#[tokio::test]
async fn calls() {
    let result = post(
        greeter(),
        "/services.calls.Greeter/greet",
        r#"{"name":"Ada"}"#,
    )
    .await;
    assert_eq!(
        result,
        (StatusCode::OK, r#"{"greeting":"Hello, Ada!"}"#.to_string())
    );

    let result = post(
        greeter(),
        "/services.calls.Greeter/step",
        r#"{"x":1,"y":2}"#,
    )
    .await;
    assert_eq!(result, (StatusCode::OK, r#"{"x":2,"y":3}"#.to_string()));

    let result = post(greeter(), "/services.calls.Greeter/color", "2").await;
    assert_eq!(result, (StatusCode::OK, r#""Green""#.to_string()));

    let result = post(greeter(), "/services.calls.Greeter/color", "4").await;
    assert_eq!(result, (StatusCode::OK, "4".to_string()));
}

#[tokio::test]
async fn errors() {
    let result = post(greeter(), "/services.calls.Greeter/greet", "{}").await;
    assert_eq!(
        result,
        (
            StatusCode::BAD_REQUEST,
            r#"{"reason":"BadRequest"}"#.to_string()
        )
    );

    let result = post(greeter(), "/services.calls.Greeter/color", "0").await;
    assert_eq!(
        result,
        (
            StatusCode::NOT_FOUND,
            r#"{"reason":"NotFound"}"#.to_string()
        )
    );

    let (status, _) = post(greeter(), "/services.calls.Greeter/unknown", "{}").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn keyword_calls() {
    let router: Router = keyword_service_router(Arc::new(TestKeywordService));
    let result = post(router.clone(), "/names.keywords.KeywordService/type", "1").await;
    assert_eq!(result, (StatusCode::OK, "2".to_string()));

    let result = post(router, "/names.keywords.KeywordService/self", "1").await;
    assert_eq!(result, (StatusCode::OK, "3".to_string()));
}

#[tokio::test]
async fn invalid_bodies() {
    let bad_request = (
        StatusCode::BAD_REQUEST,
        r#"{"reason":"BadRequest"}"#.to_string(),
    );
    let result = post(greeter(), "/services.calls.Greeter/greet", "{").await;
    assert_eq!(result, bad_request);
    let result = post(greeter(), "/services.calls.Greeter/greet", r#"{"name":1}"#).await;
    assert_eq!(result, bad_request);

    let request: Request<Body> = Request::post("/services.calls.Greeter/greet")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(
        send(greeter(), request).await,
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            r#"{"reason":"UnsupportedMediaType"}"#.to_string()
        )
    );
}
