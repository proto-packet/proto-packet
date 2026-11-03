mod common;

use common::{ID, ID_JSON, assert_json};
use proto_packet_test::structs::boxed::{BoxedFields, ListNode, OptionalBoxedFields};
use proto_packet_test::structs::copy::{BoxedCopy, CopyFields, NestedCopy, OptionalCopy};
use proto_packet_test::structs::empty::EmptyStruct;
use proto_packet_test::structs::fields::{NamedFields, PrimitiveFields, SpecialFields};
use proto_packet_test::structs::optional::OptionalFields;
use proto_packet_test::types::{Color, Label, Person, Point, Shape};

/// Asserts the type `T` is `Copy`.
fn assert_copy<T: Copy>() {}

/// Gets the test person.
fn person() -> Person {
    Person::new("Ada".to_string(), 36)
}

#[test]
fn copy() {
    assert_copy::<Point>();
    assert_copy::<CopyFields>();
    assert_copy::<NestedCopy>();
    assert_copy::<OptionalCopy>();
}

#[test]
fn fields_json() {
    assert_json(
        &PrimitiveFields::new(true, 1, 2, 3, 4, u128::MAX),
        &format!(
            r#"{{"boolean":true,"unsigned8":1,"unsigned16":2,"unsigned32":3,"unsigned64":4,"unsigned128":{}}}"#,
            u128::MAX
        ),
    );
    assert_json(
        &SpecialFields::new(ID, "text".to_string()),
        &format!(r#"{{"id":{},"text":"text"}}"#, ID_JSON),
    );
    assert_json(
        &NamedFields::new(
            Point::new(1, 2),
            person(),
            Label::default().with_text("label".to_string()),
            Color::Green,
            Shape::Circle(3),
        ),
        r#"{"point":{"x":1,"y":2},"person":{"name":"Ada","age":36},"label":{"text":"label"},"color":"Green","shape":{"var":"Circle","val":3}}"#,
    );
}

#[test]
fn optional_fields_json() {
    let empty: OptionalFields = OptionalFields::new(None, None, None, None, None, None, None, None);
    assert_json(&empty, "{}");

    let some: OptionalFields = empty
        .with_count(1)
        .with_text("text".to_string())
        .with_color(Color::Blue);
    assert_json(&some, r#"{"count":1,"text":"text","color":"Blue"}"#);
}

#[test]
fn boxed_fields_json() {
    let boxed: BoxedFields = BoxedFields::new(
        Point::new(1, 2),
        person(),
        Label::default(),
        Color::Red,
        Shape::Square(Point::new(3, 4)),
    );
    assert_json(
        &boxed,
        r#"{"point":{"x":1,"y":2},"person":{"name":"Ada","age":36},"label":{},"color":"Red","shape":{"var":"Square","val":{"x":3,"y":4}}}"#,
    );

    let list: ListNode = ListNode::new(1, Some(ListNode::new(2, None)));
    assert_json(&list, r#"{"value":1,"next":{"value":2}}"#);
    assert_eq!(list.next().map(ListNode::value), Some(2));
}

#[test]
fn empty_json() {
    assert_json(&EmptyStruct {}, "{}");
}

#[test]
fn setters_return_the_previous_value() {
    let mut special: SpecialFields = SpecialFields::new(ID, "old".to_string());
    assert_eq!(special.set_text("new".to_string()), "old");
    assert_eq!(special.text(), "new");

    let mut copy: CopyFields = CopyFields::new(1, ID, Point::new(1, 2), Color::Red);
    assert_eq!(copy.set_point(Point::new(3, 4)), Point::new(1, 2));
    assert_eq!(copy.point(), Point::new(3, 4));

    let mut optional: OptionalFields =
        OptionalFields::new(None, None, None, None, None, None, None, None);
    assert_eq!(optional.set_person(person()), None);
    assert_eq!(optional.set_person(None), Some(person()));
    assert_eq!(optional.person(), None);

    let mut boxed: OptionalBoxedFields = OptionalBoxedFields::new(None, None, None, None, None);
    assert_eq!(boxed.set_person(person()), None);
    assert_eq!(boxed.person(), Some(&person()));
    assert_eq!(boxed.set_person(None), Some(person()));
}

#[test]
fn const_constructors() {
    const POINT: Point = Point::new(1, 2);
    const OPTIONAL: OptionalCopy = OptionalCopy::new(Some(1), None);
    assert_eq!(POINT.x(), 1);
    assert_eq!(OPTIONAL.count(), Some(1));
}

#[test]
fn new_from_converts_the_values() {
    assert_eq!(
        SpecialFields::new_from(ID, "text"),
        SpecialFields::new(ID, "text".to_string())
    );
    assert_eq!(
        OptionalCopy::new_from(1, None),
        OptionalCopy::new(Some(1), None)
    );
    assert_eq!(
        BoxedCopy::new_from(Point::new(1, 2)).point(),
        Point::new(1, 2)
    );
}
