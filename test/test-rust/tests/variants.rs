mod common;

use common::{ID, ID_JSON, assert_json};
use proto_packet_test::types::{Color, Label, Person, Point, Shape};
use proto_packet_test::variants::boxed::{Expr, Sum};
use proto_packet_test::variants::cases::Value;

#[test]
fn cases_json() {
    assert_json(&Value::Boolean(true), r#"{"var":"Boolean","val":true}"#);
    assert_json(&Value::Unsigned64(1), r#"{"var":"Unsigned64","val":1}"#);
    assert_json(
        &Value::Id(ID),
        &format!(r#"{{"var":"Id","val":{}}}"#, ID_JSON),
    );
    assert_json(
        &Value::Text("text".to_string()),
        r#"{"var":"Text","val":"text"}"#,
    );
    assert_json(
        &Value::Point(Point::new(1, 2)),
        r#"{"var":"Point","val":{"x":1,"y":2}}"#,
    );
    assert_json(
        &Value::Person(Person::new("Ada".to_string(), 36)),
        r#"{"var":"Person","val":{"name":"Ada","age":36}}"#,
    );
    assert_json(
        &Value::Label(Label::default().with_text("label".to_string())),
        r#"{"var":"Label","val":{"text":"label"}}"#,
    );
    assert_json(&Value::Color(Color::Red), r#"{"var":"Color","val":"Red"}"#);
    assert_json(
        &Value::Shape(Shape::Circle(1)),
        r#"{"var":"Shape","val":{"var":"Circle","val":1}}"#,
    );
}

#[test]
fn boxed_json() {
    let expr: Expr = Expr::Add(Sum::new(
        Expr::Number(1),
        Expr::Negate(Box::new(Expr::Number(2))),
    ));
    assert_json(
        &expr,
        r#"{"var":"Add","val":{"left":{"var":"Number","val":1},"right":{"var":"Negate","val":{"var":"Number","val":2}}}}"#,
    );
}

#[test]
fn variants_are_ordered_by_tag_number() {
    use proto_packet_test::variants::cases::UnorderedTags;

    assert!(UnorderedTags::One(9) < UnorderedTags::Two(0));
    assert!(UnorderedTags::Two(9) < UnorderedTags::Three(0));
    assert!(UnorderedTags::One(1) < UnorderedTags::One(2));
}
