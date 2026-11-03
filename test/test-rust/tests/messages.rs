mod common;

use common::{ID, ID_JSON, assert_json};
use proto_packet_test::messages::boxed::{BoxedFields, Comment};
use proto_packet_test::messages::empty::EmptyMessage;
use proto_packet_test::messages::fields::{NamedFields, PrimitiveFields, SpecialFields};
use proto_packet_test::types::{Color, Label, Person, Point, Shape};

#[test]
fn default_json() {
    let fields: PrimitiveFields = PrimitiveFields::default();
    assert_eq!(fields.boolean(), None);
    assert_eq!(fields.unsigned128(), None);
    assert_json(&fields, "{}");
    assert_json(&EmptyMessage::default(), "{}");
}

#[test]
fn fields_json() {
    assert_json(
        &PrimitiveFields::default()
            .with_boolean(false)
            .with_unsigned8(1)
            .with_unsigned128(u128::MAX),
        &format!(
            r#"{{"boolean":false,"unsigned8":1,"unsigned128":{}}}"#,
            u128::MAX
        ),
    );
    assert_json(
        &SpecialFields::default()
            .with_id(ID)
            .with_text("text".to_string()),
        &format!(r#"{{"id":{},"text":"text"}}"#, ID_JSON),
    );
    assert_json(
        &NamedFields::default()
            .with_point(Point::new(1, 2))
            .with_person(Person::new("Ada".to_string(), 36))
            .with_label(Label::default())
            .with_color(Color::Red)
            .with_shape(Shape::Circle(3)),
        r#"{"point":{"x":1,"y":2},"person":{"name":"Ada","age":36},"label":{},"color":"Red","shape":{"var":"Circle","val":3}}"#,
    );
}

#[test]
fn boxed_json() {
    let comment: Comment = Comment::default()
        .with_text("first".to_string())
        .with_reply(Comment::default().with_text("second".to_string()));
    assert_json(&comment, r#"{"text":"first","reply":{"text":"second"}}"#);
    assert_eq!(comment.reply().and_then(Comment::text), Some("second"));
}

#[test]
fn setters_return_the_previous_value() {
    let mut fields: SpecialFields = SpecialFields::default();
    assert_eq!(fields.set_text("old".to_string()), None);
    assert_eq!(fields.set_text("new".to_string()), Some("old".to_string()));
    assert_eq!(fields.set_text(None), Some("new".to_string()));
    assert_eq!(fields.text(), None);

    let mut comment: Comment = Comment::default();
    let reply: Comment = Comment::default().with_text("reply".to_string());
    assert_eq!(comment.set_reply(reply.clone()), None);
    assert_eq!(comment.set_reply(None), Some(reply));
}

#[test]
fn boxed_copy_fields() {
    let mut boxed: BoxedFields = BoxedFields::default().with_point(Point::new(1, 2));
    assert_eq!(boxed.point(), Some(Point::new(1, 2)));
    assert_eq!(boxed.set_point(None), Some(Point::new(1, 2)));
    assert_eq!(boxed.point(), None);
}
