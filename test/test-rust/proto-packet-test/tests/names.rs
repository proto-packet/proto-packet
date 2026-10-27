mod common;

use common::*;
use proto_packet::axum::Router;
use proto_packet::axum::http::StatusCode;
use proto_packet::service::ServiceError;
use proto_packet_test::names::casing::{
    HTTPServer, My_Enum, My_Message, My_Service, My_Struct, My_Variant, Vec3, my_service_router,
};
use proto_packet_test::names::keywords::{
    KeywordEnum, KeywordMessage, KeywordService, KeywordStruct, KeywordVariant, Self_, Type,
    keyword_service_router,
};
use proto_packet_test::names::r#move::MoveStruct;
use proto_packet_test::names::myMod::CamelStruct;
use proto_packet_test::names::prelude;
use proto_packet_test::names::r#type::self_::SelfStruct;
use std::sync::Arc;

#[test]
fn keyword_fields() {
    let mut value: KeywordStruct = KeywordStruct::new(1, "match".to_string(), 2);
    assert_eq!(value.r#type(), 1);
    assert_eq!(value.r#match(), "match");
    assert_eq!(value.self_(), 2);

    let result: u32 = value.set_type(3);
    assert_eq!(result, 1);
    let result: u32 = value.set_self(4);
    assert_eq!(result, 2);
    let value: KeywordStruct = value.with_match("other".to_string());
    assert_eq!(value, KeywordStruct::new(3, "other".to_string(), 4));

    let value: KeywordMessage = KeywordMessage::default().with_type(1_u32).with_self(2_u32);
    assert_eq!(value.r#type(), Some(1));
    assert_eq!(value.self_(), Some(2));
}

#[test]
fn keyword_fields_json_keeps_the_schema_names() {
    assert_json(
        &KeywordStruct::new(1, "match".to_string(), 2),
        r#"{"type":1,"match":"match","self":2}"#,
    );
    assert_json(
        &KeywordMessage::default().with_type(1_u32).with_self(2_u32),
        r#"{"type":1,"self":2}"#,
    );
}

#[test]
fn keyword_cases() {
    let result: KeywordEnum = KeywordEnum::Self_;
    assert_eq!(result, KeywordEnum::Self_);
    let result: KeywordVariant = KeywordVariant::Self_(1);
    assert_eq!(result, KeywordVariant::Self_(1));
}

#[test]
fn keyword_types() {
    assert_eq!(Type::new(1).value(), 1);
    assert_eq!(Self_::new(1).value(), 1);
}

/// A service with keyword calls.
struct TestKeywordService;

impl KeywordService for TestKeywordService {
    async fn r#type(&self, request: u32) -> Result<u32, ServiceError> {
        Ok(request + 1)
    }

    async fn self_(&self, request: u32) -> Result<u32, ServiceError> {
        Ok(request + 2)
    }
}

#[tokio::test]
async fn keyword_calls_are_routed_by_their_schema_names() {
    let test_cases: &[(&str, &str)] = &[("type", "2"), ("self", "3")];
    for (call, expected) in test_cases {
        let path: String = format!("/names.keywords.KeywordService/{}", call);
        let result: (StatusCode, String) = post(
            keyword_service_router(Arc::new(TestKeywordService)),
            &path,
            "1",
        )
        .await;
        assert_eq!(
            result,
            (StatusCode::OK, expected.to_string()),
            "call: {}",
            call
        );
    }
}

#[test]
fn keyword_mods() {
    assert_eq!(MoveStruct::new(1).value(), 1);
    assert_eq!(SelfStruct::new(1).value(), 1);
}

#[test]
fn non_snake_case_mods() {
    assert_eq!(CamelStruct::new(1).value(), 1);
}

#[test]
fn non_standard_casing() {
    let mut value: My_Struct = My_Struct::new(1);
    assert_eq!(value.myField(), 1);
    let result: u32 = value.set_myField(2);
    assert_eq!(result, 1);
    assert_json(&value, r#"{"myField":2}"#);

    let value: My_Message = My_Message::default().with_myField(1_u32);
    assert_json(&value, r#"{"myField":1}"#);

    assert_json(&My_Enum::Case_One, r#""Case_One""#);
    assert_json(&My_Enum::CaseTwo, r#""CaseTwo""#);
    assert_json(&My_Variant::Case_One(1), r#"{"var":"Case_One","val":1}"#);
    assert_json(&My_Variant::CaseTwo(2), r#"{"var":"CaseTwo","val":2}"#);

    assert_json(
        &HTTPServer::new("https://example.com".to_string()),
        r#"{"url":"https://example.com"}"#,
    );
    assert_json(&Vec3::new(1.0, 2.0, 3.0), r#"{"x":1.0,"y":2.0,"z":3.0}"#);
}

/// A service with a non-camel-case name.
struct TestMyService;

impl My_Service for TestMyService {
    async fn call(&self, request: u32) -> Result<u32, ServiceError> {
        Ok(request)
    }
}

#[tokio::test]
async fn non_camel_case_services_are_routed_by_their_schema_names() {
    let result: (StatusCode, String) = post(
        my_service_router(Arc::new(TestMyService)),
        "/names.casing.My_Service/call",
        "1",
    )
    .await;
    assert_eq!(result, (StatusCode::OK, "1".to_string()));
}

#[test]
fn prelude_names() {
    let option: prelude::Option = prelude::Option::new(Some(1));
    assert_eq!(option.value(), Some(1));
    let some: prelude::Some = prelude::Some::new(Some("text".to_string()));
    assert_eq!(some.value(), Some("text"));
    let none: prelude::None = prelude::None::new(None);
    assert_eq!(none.value(), None);
    let string: prelude::String = prelude::String::new("text".to_string());
    assert_eq!(string.text(), "text");
    let vec: prelude::Vec = prelude::Vec::new(vec![1, 2]);
    assert_eq!(vec.values(), [1, 2]);
    let boxed: prelude::Box = prelude::Box::new(point());
    assert_eq!(boxed.point(), point());
    let default: prelude::Default = prelude::Default::default().with_value(1_u32);
    assert_eq!(default.value(), Some(1));
    assert!(prelude::Ordering::Less < prelude::Ordering::Greater);

    let value: prelude::PreludeFields = prelude::PreludeFields::new(
        option,
        string,
        vec,
        boxed,
        prelude::Ordering::Less,
        prelude::Result::Err("error".to_string()),
    );
    assert_json(
        &value,
        &format!(
            concat!(
                r#"{{"option":{{"value":1}},"string":{{"text":"text"}},"vec":{{"values":[1,2]}},"#,
                r#""box":{{"point":{}}},"ordering":"Less","result":{{"var":"Err","val":"error"}}}}"#
            ),
            POINT_JSON
        ),
    );
}

/// The services named like prelude traits.
struct TestPreludeServices;

impl prelude::Future for TestPreludeServices {
    async fn call(&self, request: u32) -> Result<u32, ServiceError> {
        Ok(request)
    }
}

impl prelude::Send for TestPreludeServices {
    async fn call(&self, request: u32) -> Result<u32, ServiceError> {
        Ok(request)
    }
}

impl prelude::Sync for TestPreludeServices {
    async fn call(&self, request: u32) -> Result<u32, ServiceError> {
        Ok(request)
    }
}

#[tokio::test]
async fn prelude_named_services() {
    let services: Arc<TestPreludeServices> = Arc::new(TestPreludeServices);
    let test_cases: Vec<(Router, &str)> = vec![
        (
            prelude::future_router(services.clone()),
            "/names.prelude.Future/call",
        ),
        (
            prelude::send_router(services.clone()),
            "/names.prelude.Send/call",
        ),
        (prelude::sync_router(services), "/names.prelude.Sync/call"),
    ];
    for (router, path) in test_cases {
        let result: (StatusCode, String) = post(router, path, "1").await;
        assert_eq!(result, (StatusCode::OK, "1".to_string()), "path: {}", path);
    }
}
