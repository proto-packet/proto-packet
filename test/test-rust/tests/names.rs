mod common;

use common::assert_json;
use proto_packet_test::names::casing::{HTTPServer, Log_Level};
use proto_packet_test::names::keywords::{
    KeywordEnum, KeywordMessage, KeywordStruct, KeywordVariant,
};
use proto_packet_test::names::methods::DefaultField;

#[test]
fn keywords_json() {
    let structure: KeywordStruct = KeywordStruct::new(1, "match".to_string(), 2);
    assert_eq!(structure.r#type(), 1);
    assert_eq!(structure.self_(), 2);
    assert_json(&structure, r#"{"type":1,"match":"match","self":2}"#);

    let message: KeywordMessage = KeywordMessage::default().with_type(1).with_self(2);
    assert_json(&message, r#"{"type":1,"self":2}"#);

    assert_json(&KeywordEnum::Self_, r#""Self""#);
    assert_json(&KeywordVariant::Self_(1), r#"{"var":"Self","val":1}"#);
}

#[test]
fn casing_json() {
    let server: HTTPServer = HTTPServer::new(1);
    assert_eq!(server.maxConnections(), 1);
    assert_json(&server, r#"{"maxConnections":1}"#);

    assert_json(&Log_Level::INFO, r#""INFO""#);
    assert_json(&Log_Level::Very_Verbose, r#""Very_Verbose""#);
}

#[test]
fn constructor_names() {
    assert_eq!(DefaultField::new(1).default(), 1);
}
