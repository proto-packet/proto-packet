mod common;

use common::assert_json;
use proto_packet_test::names::casing::{HTTPServer, Log_Level};
use proto_packet_test::names::keywords::{
    KeywordEnum, KeywordMessage, KeywordStruct, KeywordVariant,
};
use proto_packet_test::names::methods::{
    DefaultField, MethodCases, MethodFields, MethodFieldsBuf, MethodFieldsRef, MethodFieldsView,
    MethodMessage, MethodNest, MethodNestRef, MethodNestView, MethodTexts,
};
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Gets the test struct with fields named like trait methods, that starts with the `first` value.
fn method_fields(first: u32) -> MethodFields {
    MethodFields::new(first, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14)
}

/// Gets the test struct with text fields named like trait methods.
fn method_texts() -> MethodTexts {
    let [a, b, c, d, e, f]: [String; 6] = ["a", "b", "c", "d", "e", "f"].map(String::from);
    MethodTexts::new(a, b, c, d, e, f)
}

/// Gets the test message with fields named like trait methods.
fn method_message() -> MethodMessage {
    MethodMessage::default()
        .with_clone(1)
        .with_cmp(2)
        .with_into("x".to_string())
        .with_try_into(4)
}

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

#[test]
fn method_names_are_getters() {
    let fields: MethodFields = method_fields(1);
    assert_eq!(fields.clone(), 1);
    assert_eq!(fields.eq(), 2);
    assert_eq!(fields.cmp(), 3);

    // A trait method that takes the value hides the getter, so the getter is called by its path.
    assert_eq!(MethodFields::max(&fields), 4);
    assert_eq!(MethodFields::into(&fields), 9);
    assert_eq!(MethodFields::try_into(&fields), 10);

    let texts: MethodTexts = method_texts();
    assert_eq!(texts.clone(), "a");
    assert_eq!(texts.as_ref(), "b");
    assert_eq!(texts.borrow(), "c");
    assert_eq!(texts.deref(), "d");
    assert_eq!(texts.to_string(), "f");

    let message: MethodMessage = method_message();
    assert_eq!(message.clone(), Some(1));
    assert_eq!(MethodMessage::into(&message), Some("x"));
}

#[test]
fn method_names_keep_the_traits() {
    let (low, high): (MethodFields, MethodFields) = (method_fields(1), method_fields(2));
    assert_eq!(Clone::clone(&low), low);
    assert_eq!(Ord::cmp(&low, &high), Ordering::Less);
    assert_eq!(Ord::max(low, high), high);
    assert!(PartialEq::ne(&low, &high));

    let message: MethodMessage = method_message();
    assert_eq!(Clone::clone(&message), message);
    assert_eq!(
        Ord::cmp(&message, &MethodMessage::default()),
        Ordering::Greater
    );
    assert_eq!(
        Ord::cmp(&MethodCases::Fields(low), &MethodCases::Fields(high)),
        Ordering::Less
    );
}

#[test]
fn method_names_json() {
    assert_json(
        &method_fields(1),
        r#"{"clone":1,"eq":2,"cmp":3,"max":4,"min":5,"hash":6,"fmt":7,"from":8,"into":9,"try_into":10,"to_owned":11,"as_ref":12,"into_iter":13,"from_str":14}"#,
    );
    assert_json(
        &method_texts(),
        r#"{"clone":"a","as_ref":"b","borrow":"c","deref":"d","into":"e","to_string":"f"}"#,
    );
    assert_json(
        &method_message(),
        r#"{"clone":1,"cmp":2,"into":"x","try_into":4}"#,
    );
    assert_json(
        &MethodCases::Message(method_message()),
        r#"{"var":"Message","val":{"clone":1,"cmp":2,"into":"x","try_into":4}}"#,
    );
}

#[test]
fn method_names_encode() {
    let nest: MethodNest = MethodNest::default()
        .with_fields(method_fields(1))
        .with_texts(method_texts())
        .with_message(method_message())
        .with_list(vec![method_message(), MethodMessage::default()])
        .with_keyed(BTreeMap::from([
            (method_fields(2), method_texts()),
            (method_fields(1), method_texts()),
        ]));
    let bytes: Vec<u8> = nest.encode();
    assert_eq!(MethodNest::decode(&bytes).unwrap(), nest);

    let nest_ref: MethodNestRef<'_> = MethodNestRef::new(&bytes).unwrap();
    assert_eq!(MethodNestView::encode(&nest_ref), bytes);
    assert_eq!(MethodNestView::encode(&&nest), bytes);

    let fields: MethodFieldsRef<'_> = nest_ref.fields().unwrap();
    assert_eq!(MethodFieldsRef::into(&fields), 9);
    assert_eq!(MethodFieldsView::try_into(&&fields), 10);

    let buf: MethodFieldsBuf = <MethodFieldsBuf as From<&MethodFields>>::from(&method_fields(1));
    assert_eq!(MethodFieldsView::into(&buf), 9);
    assert_eq!(MethodFieldsView::clone(&buf), 1);

    let cases: MethodCases = MethodCases::Texts(method_texts());
    assert_eq!(MethodCases::decode(&cases.encode()).unwrap(), cases);
}
