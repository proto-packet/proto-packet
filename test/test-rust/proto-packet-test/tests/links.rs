mod common;

use common::*;
use proto_packet_test::links::imports::ImportedFields;
use proto_packet_test::links::local::{After, Before};
use proto_packet_test::links::loops::{Category, Expr, Folder, ListNode, Sum, TreeNode};
use proto_packet_test::links::qualified::{Local, QualifiedFields};
use proto_packet_test::mods::parent::child::Child;
use proto_packet_test::names::r#move::MoveStruct;
use proto_packet_test::names::r#type::self_::SelfStruct;

#[test]
fn local_names() {
    let value: Before = Before::new(After::new(1));
    assert_eq!(value.after(), After::new(1));
}

#[test]
fn imported_names() {
    let value: ImportedFields = ImportedFields::new(
        point(),
        Child::new(1),
        MoveStruct::new(2),
        SelfStruct::new(3),
    );
    assert_eq!(value.point(), point());
    assert_eq!(value.child(), Child::new(1));
    assert_eq!(value.raw(), MoveStruct::new(2));
    assert_eq!(value.non_raw(), SelfStruct::new(3));
}

#[test]
fn qualified_names() {
    let value: QualifiedFields = QualifiedFields::new(
        point(),
        Child::new(1),
        MoveStruct::new(2),
        SelfStruct::new(3),
        Local::new(4),
    );
    assert_eq!(value.point(), point());
    assert_eq!(value.child(), Child::new(1));
    assert_eq!(value.raw(), MoveStruct::new(2));
    assert_eq!(value.non_raw(), SelfStruct::new(3));
    assert_eq!(value.local(), Local::new(4));
}

#[test]
fn struct_loops_through_optional_boxes() {
    let value: ListNode = ListNode::new(1, Some(ListNode::new(2, Some(ListNode::new(3, None)))));
    let mut result: Vec<u32> = Vec::default();
    let mut node: Option<&ListNode> = Some(&value);
    while let Some(current) = node {
        result.push(current.value());
        node = current.next();
    }
    assert_eq!(result, [1, 2, 3]);
    assert_json(
        &value,
        r#"{"value":1,"next":{"value":2,"next":{"value":3}}}"#,
    );
}

#[test]
fn struct_loops_through_slices() {
    let value: TreeNode =
        TreeNode::new(1, vec![TreeNode::new(2, vec![]), TreeNode::new(3, vec![])]);
    let result: Vec<u32> = value.children().iter().map(TreeNode::value).collect();
    assert_eq!(result, [2, 3]);
    assert_json(
        &value,
        r#"{"value":1,"children":[{"value":2,"children":[]},{"value":3,"children":[]}]}"#,
    );
}

#[test]
fn message_loops_through_boxes() {
    let value: Category = Category::default()
        .with_name("child".to_string())
        .with_parent(Category::default().with_name("root".to_string()));
    assert_eq!(value.parent().and_then(Category::name), Some("root"));
    assert_json(&value, r#"{"name":"child","parent":{"name":"root"}}"#);
}

#[test]
fn message_loops_through_slices() {
    let value: Folder = Folder::default()
        .with_name("root".to_string())
        .with_folders(vec![Folder::default().with_name("child".to_string())]);
    assert_json(&value, r#"{"name":"root","folders":[{"name":"child"}]}"#);
}

/// Evaluates the `expr`.
fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Literal(value) => *value,
        Expr::Negate(expr) => -eval(expr),
        Expr::Add(sum) => eval(sum.left()) + eval(sum.right()),
    }
}

#[test]
fn variant_loops_through_boxes() {
    let value: Expr = Expr::Add(Box::new(Sum::new(
        Expr::Literal(1),
        Expr::Negate(Box::new(Expr::Literal(3))),
    )));
    assert_eq!(eval(&value), -2);
    assert_json(
        &value,
        concat!(
            r#"{"var":"Add","val":{"left":{"var":"Literal","val":1},"#,
            r#""right":{"var":"Negate","val":{"var":"Literal","val":3}}}}"#
        ),
    );
}
