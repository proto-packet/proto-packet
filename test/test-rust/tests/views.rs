//! The tests of the encoding of any form of a type, from the getters of its `View` trait.

use proto_packet::io::{CopySlice, DecodeError, ListView, ViewSlice};
use proto_packet_test::format::examples::{
    Entry, EntryBuf, EntryMut, EntryRef, EntryView, Point, PointView,
};
use proto_packet_test::format::lists::{WithLists, WithListsRef, WithListsView};
use proto_packet_test::format::messages::{Newer, NewerRef, NewerView, Older, OlderRef, OlderView};
use proto_packet_test::format::variants::{
    NewerCases, OlderCases, OlderCasesRef, OlderCasesView, Shape, ShapeView,
};
use proto_packet_test::messages::boxed::{Comment, CommentRef, CommentView};

/// An entry of borrowed data, with no native entry.
#[derive(Copy, Clone)]
struct Row<'a> {
    id: u64,
    text: &'a str,
}

impl EntryView for Row<'_> {
    fn id(&self) -> u64 {
        self.id
    }

    fn text(&self) -> &str {
        self.text
    }
}

/// A message of borrowed data, with a nested view.
struct Profile<'a> {
    name: &'a str,
    row: Row<'a>,
}

impl NewerView for Profile<'_> {
    fn id(&self) -> Option<u64> {
        Some(1)
    }

    fn name(&self) -> Option<&str> {
        Some(self.name)
    }

    fn nickname(&self) -> Option<&str> {
        None
    }

    fn age(&self) -> Option<u32> {
        Some(36)
    }

    fn entry(&self) -> Option<impl EntryView> {
        Some(self.row)
    }
}

/// A message of borrowed lists.
struct Table<'a> {
    texts: &'a [&'a str],
    rows: &'a [Row<'a>],
}

impl WithListsView for Table<'_> {
    fn bytes(&self) -> Option<&[u8]> {
        None
    }

    fn numbers(&self) -> Option<impl ListView<Item = u32>> {
        Some(&[7_u32, 8][..])
    }

    fn texts(&self) -> Option<impl ListView<Item = &str>> {
        Some(CopySlice::new(self.texts))
    }

    fn points(&self) -> Option<impl ListView<Item = impl PointView>> {
        None::<&[Point]>
    }

    fn entries(&self) -> Option<impl ListView<Item = impl EntryView>> {
        Some(ViewSlice::new(self.rows))
    }

    fn children(&self) -> Option<impl ListView<Item = impl WithListsView>> {
        None::<&[WithLists]>
    }
}

/// A circle of a shape, with no native shape.
struct Circle {
    radius: u32,
}

impl ShapeView for Circle {
    fn tag_number(&self) -> u16 {
        1
    }

    fn as_circle(&self) -> Option<u32> {
        Some(self.radius)
    }

    fn as_square(&self) -> Option<impl PointView> {
        None::<Point>
    }
}

/// A comment of borrowed data, with a reply of any form.
struct Thread<'a, R> {
    text: &'a str,
    reply: Option<R>,
}

impl<R> CommentView for Thread<'_, R>
where
    R: CommentView,
{
    fn text(&self) -> Option<&str> {
        Some(self.text)
    }

    fn reply(&self) -> Option<impl CommentView> {
        self.reply.as_ref()
    }
}

/// Gets the `count` of comments, each the reply of the one before it.
fn comments(count: usize) -> Comment {
    (1..count).fold(Comment::default().with_text("x".to_string()), |reply, _| {
        Comment::default().with_reply(reply)
    })
}

#[test]
fn borrowed_data_is_encoded_with_no_native_value() {
    let row: Row = Row { id: 7, text: "hi" };
    let entry: Entry = Entry::new(7, "hi".to_string());
    assert_eq!(row.encode(), entry.encode());
    assert_eq!(row.encoded_len(), entry.encoded_len());

    let mut out: Vec<u8> = vec![9];
    row.encode_to(&mut out);
    assert_eq!(out[1..], entry.encode());

    let profile: Profile = Profile { name: "Ada", row };
    let newer: Newer = Newer::default()
        .with_id(1)
        .with_name("Ada".to_string())
        .with_age(36)
        .with_entry(entry);
    assert_eq!(profile.encode(), newer.encode());
    assert_eq!(profile.encoded_len(), newer.encoded_len());
    assert_eq!(NewerRef::new(&profile.encode()).unwrap(), newer);

    assert_eq!(Circle { radius: 5 }.encode(), Shape::Circle(5).encode());
}

#[test]
fn borrowed_lists_are_encoded_with_no_native_list() {
    let rows: [Row; 3] = [
        Row { id: 1, text: "a" },
        Row { id: 2, text: "" },
        Row { id: 3, text: "abc" },
    ];
    let table: Table = Table {
        texts: &["x", "", "yz"],
        rows: &rows,
    };
    let native: WithLists = WithLists::default()
        .with_numbers(vec![7, 8])
        .with_texts(vec!["x".to_string(), String::default(), "yz".to_string()])
        .with_entries(
            rows.iter()
                .map(|row| Entry::new(row.id, row.text.to_string()))
                .collect::<Vec<Entry>>(),
        );
    assert_eq!(table.encode(), native.encode());
    assert_eq!(table.encoded_len(), native.encoded_len());
    assert_eq!(WithListsRef::new(&table.encode()).unwrap(), native);
}

#[test]
fn every_form_is_encoded_through_its_view() {
    /// Encodes any form of an entry.
    fn encode<V: EntryView>(view: &V) -> (usize, Vec<u8>) {
        (view.encoded_len(), view.encode())
    }

    let entry: Entry = Entry::new(7, "hi".to_string());
    let mut bytes: Vec<u8> = entry.encode();
    let expected: (usize, Vec<u8>) = (bytes.len(), bytes.clone());
    assert_eq!(encode(&entry), expected);
    assert_eq!(encode(&&entry), expected);
    assert_eq!(encode(&EntryRef::new(&bytes).unwrap()), expected);
    assert_eq!(encode(&EntryBuf::from(&entry)), expected);
    assert_eq!(encode(&EntryMut::new(&mut bytes).unwrap()), expected);
}

#[test]
fn generated_forms_keep_their_unknown_fields_and_cases() {
    let bytes: Vec<u8> = Newer::default()
        .with_name("Ada".to_string())
        .with_nickname("A".to_string())
        .encode();
    let older: Older = Older::decode(&bytes).unwrap();
    assert_eq!(OlderView::encode(&older), bytes);
    assert_eq!(OlderView::encode(&OlderRef::new(&bytes).unwrap()), bytes);

    let bytes: Vec<u8> = NewerCases::Number(5).encode();
    let older: OlderCases = OlderCases::decode(&bytes).unwrap();
    assert_eq!(OlderCasesView::encode(&older), bytes);
    assert_eq!(
        OlderCasesView::encode(&OlderCasesRef::new(&bytes).unwrap()),
        bytes
    );
}

#[test]
fn views_hold_any_form_of_their_values() {
    let reply: Vec<u8> = comments(2).encode();
    let thread = Thread {
        text: "top",
        reply: Some(CommentRef::new(&reply).unwrap()),
    };
    let native: Comment = Comment::default()
        .with_text("top".to_string())
        .with_reply(comments(2));
    assert_eq!(thread.encode(), native.encode());
    assert_eq!(thread.encoded_len(), native.encoded_len());

    // an encoded reply that fits inside one more level
    let deepest: Vec<u8> = comments(63).encode();
    let thread = Thread {
        text: "top",
        reply: Some(CommentRef::new(&deepest).unwrap()),
    };
    assert!(Comment::decode(&thread.encode()).is_ok());
}

#[test]
#[should_panic(expected = "the value nests deeper than 64 levels")]
fn views_refuse_encoded_values_that_nest_too_deep() {
    let deepest: Vec<u8> = comments(64).encode();
    let thread = Thread {
        text: "top",
        reply: Some(CommentRef::new(&deepest).unwrap()),
    };
    let _ = thread.encode();
}

#[test]
fn views_can_write_past_the_depth_only_when_told_to() {
    let deepest: Vec<u8> = comments(64).encode();
    let thread = Thread {
        text: "top",
        reply: Some(CommentRef::new(&deepest).unwrap()),
    };
    let mut bytes: Vec<u8> = Vec::default();
    thread.encode_to_with_depth(&mut bytes, 65);
    assert_eq!(Comment::decode(&bytes), Err(DecodeError::TooDeep));
}

/// A list that has another count of elements than its length.
#[derive(Copy, Clone)]
struct Short;

impl ListView for Short {
    type Item = u32;

    fn len(&self) -> usize {
        2
    }

    fn get(&self, _index: usize) -> Option<u32> {
        None
    }

    fn iter(&self) -> impl Iterator<Item = u32> {
        std::iter::once(1)
    }
}

/// A message with a list that has another count of elements than its length.
struct Broken;

impl WithListsView for Broken {
    fn bytes(&self) -> Option<&[u8]> {
        None
    }

    fn numbers(&self) -> Option<impl ListView<Item = u32>> {
        Some(Short)
    }

    fn texts(&self) -> Option<impl ListView<Item = &str>> {
        None::<&[String]>
    }

    fn points(&self) -> Option<impl ListView<Item = impl PointView>> {
        None::<&[Point]>
    }

    fn entries(&self) -> Option<impl ListView<Item = impl EntryView>> {
        None::<&[Entry]>
    }

    fn children(&self) -> Option<impl ListView<Item = impl WithListsView>> {
        None::<&[WithLists]>
    }
}

#[test]
#[should_panic(expected = "the list has another count than its length")]
fn views_refuse_lists_with_another_count_than_their_length() {
    let _ = Broken.encode();
}
