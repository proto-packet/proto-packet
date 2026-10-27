//! The helpers shared by the tests.

#![allow(dead_code)] // each test crate uses only some of the helpers

use proto_packet::axum::Router;
use proto_packet::axum::body::{Body, to_bytes};
use proto_packet::axum::http::{Request, StatusCode};
use proto_packet::axum::response::Response;
use proto_packet::serde::Serialize;
use proto_packet::serde::de::DeserializeOwned;
use proto_packet::types::{Date, Uuid};
use proto_packet_test::types::{Label, Path, Point, Shape};
use std::fmt::Debug;
use tower::ServiceExt;

/// Asserts the `value` serializes to the `json` & deserializes from it.
pub fn assert_json<T>(value: &T, json: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let result: String = serde_json::to_string(value).unwrap();
    assert_eq!(result, json);
    let result: T = serde_json::from_str(json).unwrap();
    assert_eq!(&result, value);
}

/// Posts the JSON `body` to the `path` of the `router`. Returns the response status & body.
pub async fn post(router: Router, path: &str, body: &str) -> (StatusCode, String) {
    let request: Request<Body> = Request::post(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    send(router, request).await
}

/// Sends the `request` to the `router`. Returns the response status & body.
pub async fn send(router: Router, request: Request<Body>) -> (StatusCode, String) {
    let response: Response = router.oneshot(request).await.unwrap();
    let status: StatusCode = response.status();
    let body: Vec<u8> = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    (status, String::from_utf8(body).unwrap())
}

/// Gets the test `uuid`.
pub fn id() -> Uuid {
    Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef)
}

/// The JSON of the test `uuid`.
pub const ID_JSON: &str = "\"01234567-89ab-cdef-0123-456789abcdef\"";

/// Gets the test `date`. (a leap day)
pub fn day() -> Date {
    Date::from_ymd_opt(2024, 2, 29).unwrap()
}

/// The JSON of the test `date`.
pub const DAY_JSON: &str = "\"2024-02-29\"";

/// Gets the test `Point`.
pub fn point() -> Point {
    Point::new(1, -2)
}

/// The JSON of the test `Point`.
pub const POINT_JSON: &str = r#"{"x":1,"y":-2}"#;

/// Gets the test `Path`.
pub fn path() -> Path {
    Path::new(vec![Point::new(1, -2), Point::new(3, 4)])
}

/// The JSON of the test `Path`.
pub const PATH_JSON: &str = r#"{"points":[{"x":1,"y":-2},{"x":3,"y":4}]}"#;

/// Gets the test `Label`.
pub fn label() -> Label {
    Label::default().with_text("label".to_string())
}

/// The JSON of the test `Label`.
pub const LABEL_JSON: &str = r#"{"text":"label"}"#;

/// Gets the test `Shape`.
pub fn shape() -> Shape {
    Shape::Circle(1.5.into())
}

/// The JSON of the test `Shape`.
pub const SHAPE_JSON: &str = r#"{"var":"Circle","val":1.5}"#;
