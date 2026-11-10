mod common;

use common::{ID, assert_json};
use proto_packet::io::{DecodeError, ListRef, ListView};
use proto_packet::types::{Bytes, Float64};
use proto_packet_test::format::examples::{Color, Entry, EntryView, Line, Point, PointView};
use proto_packet_test::format::lists::{
    Lines, ListCases, ListCasesBuf, ListCasesCase, ListCasesRef, ListCasesView, ListKinds,
    ListKindsBuf, ListKindsMut, ListKindsRef, ListKindsView, Nested, NestedRef, NestedView,
    Numbers, NumbersBuf, NumbersMut, NumbersRef, NumbersView, Texts, TextsRef, TextsView,
    WithLists, WithListsRef, WithListsView,
};
use proto_packet_test::format::messages::Item;
use proto_packet_test::format::variants::Shape;

/// Asserts the `value` of the `type` encodes to the `bytes`, & decodes back from them.
macro_rules! assert_encoding {
    ($type:ty, $value:expr, $bytes:expr) => {{
        let value: $type = $value;
        let bytes: &[u8] = $bytes;
        assert_eq!(value.encode(), bytes);
        assert_eq!(value.encoded_len(), bytes.len());
        assert_eq!(<$type>::decode(bytes), Ok(value.clone()));
        assert_eq!(unsafe { <$type>::decode_unchecked(bytes) }, value);
        common::assert_one_encoding(bytes, |changed| {
            let value: $type = <$type>::decode(changed).ok()?;
            assert_eq!(value.encoded_len(), changed.len());
            Some(value.encode())
        });
    }};
}

/// Gets the bytes of the `Numbers` example of `format/binary.md`.
fn numbers_bytes() -> Vec<u8> {
    vec![
        7, // count
        0, 0, 0, // padding
        1, 0, 0, 0, 2, 0, 0, 0, // values
    ]
}

/// Gets the bytes of the `Texts` example of `format/binary.md`.
fn texts_bytes() -> Vec<u8> {
    [
        &[3, 0, 0, 0][..],           // count
        &[14, 0, 0, 0, 14, 0, 0, 0], // ends
        b"hi",
        b"yo!",
    ]
    .concat()
}

/// Gets the value of the `Texts` example of `format/binary.md`.
fn texts() -> Texts {
    Texts::new(vec!["hi".to_string(), String::default(), "yo!".to_string()])
}

/// Gets a struct with a list of each kind of element.
fn list_kinds() -> ListKinds {
    ListKinds::new(
        Bytes::from([1, 2, 3]),
        vec![true, false],
        vec![-1, i64::MIN],
        vec![Float64::new(1.5), Float64::NAN],
        vec![ID],
        vec![Color::Red],
        vec!["a".to_string(), "bc".to_string()],
        vec![Point::new(1, 2), Point::new(3, 4)],
        vec![
            Entry::new(5, "five".to_string()),
            Entry::new(6, String::default()),
        ],
        vec![Item::default().with_id(7), Item::default()],
        vec![Shape::Circle(8), Shape::Square(Point::new(9, 10))],
        Some(vec![11, 12]),
    )
}

#[test]
fn numbers_example() {
    let bytes: Vec<u8> = numbers_bytes();
    assert_encoding!(Numbers, Numbers::new(7, vec![1, 2]), &bytes);

    let numbers: NumbersRef = NumbersRef::new(&bytes).unwrap();
    let values: ListRef<u32> = numbers.values();
    assert_eq!(values.len(), 2);
    assert_eq!(
        (values.get(0), values.get(1), values.get(2)),
        (Some(1), Some(2), None)
    );
    assert_eq!(values.iter().collect::<Vec<u32>>(), [1, 2]);
    assert_eq!(
        values.as_bytes().as_ptr() as usize % 4,
        bytes.as_ptr() as usize % 4
    );
}

#[test]
fn empty_lists_are_no_bytes() {
    assert_encoding!(Numbers, Numbers::new(7, Vec::default()), &[7]);
    assert_encoding!(Texts, Texts::new(Vec::default()), &[]);

    let numbers: NumbersRef = NumbersRef::new(&[7]).unwrap();
    assert!(numbers.values().is_empty());
    assert_eq!(numbers.values().len(), 0);
    assert_eq!(numbers.values().iter().next(), None);
}

#[test]
fn texts_example() {
    let bytes: Vec<u8> = texts_bytes();
    assert_eq!(bytes.len(), 17);
    assert_encoding!(Texts, texts(), &bytes);

    let view: TextsRef = TextsRef::new(&bytes).unwrap();
    assert_eq!(view.texts().len(), 3);
    assert_eq!(
        view.texts().iter().collect::<Vec<&str>>(),
        ["hi", "", "yo!"]
    );
    assert_eq!(view.texts().get(2), Some("yo!"));
    assert_eq!(view.texts().get(3), None);
}

#[test]
fn a_single_element_has_no_ends() {
    let bytes: [u8; 6] = [1, 0, 0, 0, b'h', b'i'];
    assert_encoding!(Texts, Texts::new(vec!["hi".to_string()]), &bytes);
}

#[test]
fn structs_are_padded_apart() {
    let line = |base: u32| Line::new(Point::new(base, base + 1), Point::new(base + 2, 9), 7);
    let bytes: Vec<u8> = [
        &[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 9, 0, 0, 0, 7][..], // the first line
        &[0; 3],                                                  // padding
        &[4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 9, 0, 0, 0, 7],     // the second line
    ]
    .concat();
    assert_eq!(bytes.len(), 37);
    assert_encoding!(Lines, Lines::new(vec![line(1), line(4)]), &bytes);

    let mut padding: Vec<u8> = bytes.clone();
    padding[18] = 0xFF;
    assert_eq!(Lines::decode(&padding), Err(DecodeError::InvalidPadding));
    assert_eq!(Lines::decode(&bytes[..36]), Err(DecodeError::InvalidList));
}

#[test]
fn lists_nest() {
    let bytes: Vec<u8> = [
        &[3, 0, 0, 0][..],           // count
        &[14, 0, 0, 0, 14, 0, 0, 0], // ends
        &[1, 2],
        &[3],
    ]
    .concat();
    let value: Nested = Nested::new(vec![
        Bytes::from([1, 2]),
        Bytes::default(),
        Bytes::from([3]),
    ]);
    assert_encoding!(Nested, value, &bytes);

    let nested: NestedRef = NestedRef::new(&bytes).unwrap();
    let rows: Vec<&[u8]> = nested.rows().iter().collect();
    assert_eq!(rows, [&[1_u8, 2][..], &[], &[3]]);
}

#[test]
fn every_kind_round_trips() {
    let value: ListKinds = list_kinds();
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(ListKinds, value.clone(), &bytes);

    let kinds: ListKindsRef = ListKindsRef::new(&bytes).unwrap();
    assert_eq!(kinds.bytes(), [1, 2, 3]);
    assert_eq!(kinds.flags().iter().collect::<Vec<bool>>(), [true, false]);
    assert_eq!(kinds.signed().get(1), Some(i64::MIN));
    assert_eq!(kinds.floats().get(1), Some(Float64::NAN));
    assert_eq!(kinds.ids().get(0), Some(ID));
    assert_eq!(kinds.colors().get(0), Some(Color::Red));
    assert_eq!(kinds.texts().get(1), Some("bc"));
    assert_eq!(kinds.points().get(1).unwrap().y(), 4);
    assert_eq!(kinds.entries().get(0).unwrap().text(), "five");
    assert_eq!(kinds.entries().get(1).unwrap().id(), 6);
    assert_eq!(kinds.items().get(0).unwrap().id(), Some(7));
    assert_eq!(kinds.shapes().get(0).unwrap().as_circle(), Some(8));
    assert_eq!(kinds.maybe().unwrap().decode(), [11, 12]);

    assert_eq!(value.points(), [Point::new(1, 2), Point::new(3, 4)]);
    assert_eq!(value.maybe(), Some(&[11_u16, 12][..]));
}

#[test]
fn absent_lists_are_not_empty_lists() {
    let absent: ListKinds = list_kinds().with_maybe(None);
    let empty: ListKinds = list_kinds().with_maybe(Vec::default());
    assert_ne!(absent.encode(), empty.encode());
    assert_eq!(absent.encoded_len(), empty.encoded_len());
    assert_eq!(ListKinds::decode(&absent.encode()).unwrap().maybe(), None);
    assert_eq!(
        ListKinds::decode(&empty.encode()).unwrap().maybe(),
        Some(&[][..])
    );
}

#[test]
fn rejects_lengths_that_do_not_fit_the_elements() {
    let mut bytes: Vec<u8> = numbers_bytes();
    bytes.push(3);
    assert_eq!(Numbers::decode(&bytes), Err(DecodeError::InvalidList));
    assert_eq!(Numbers::decode(&bytes[..10]), Err(DecodeError::InvalidList));
}

#[test]
fn rejects_invalid_counts_and_ends() {
    assert_eq!(Texts::decode(&[0, 0, 0, 0]), Err(DecodeError::InvalidList));
    assert_eq!(
        Texts::decode(&[1, 0]),
        Err(DecodeError::TooShort {
            expected: 4,
            actual: 2,
        })
    );
    assert_eq!(
        Texts::decode(&[9, 0, 0, 0, b'h', b'i']),
        Err(DecodeError::InvalidList)
    );

    let mut before_the_table: Vec<u8> = texts_bytes();
    before_the_table[4] = 11;
    assert_eq!(
        Texts::decode(&before_the_table),
        Err(DecodeError::InvalidEndTable)
    );

    let mut decreasing: Vec<u8> = texts_bytes();
    decreasing[8] = 13;
    assert_eq!(
        Texts::decode(&decreasing),
        Err(DecodeError::InvalidEndTable)
    );

    let mut past_the_end: Vec<u8> = texts_bytes();
    past_the_end[8] = 18;
    assert_eq!(
        Texts::decode(&past_the_end),
        Err(DecodeError::InvalidEndTable)
    );
}

#[test]
fn rejects_invalid_elements() {
    let mut text: Vec<u8> = texts_bytes();
    text[12] = 0xFF;
    assert_eq!(Texts::decode(&text), Err(DecodeError::InvalidUtf8));

    let flags = |flag: u8| -> Vec<u8> {
        let mut value: ListKinds = list_kinds();
        value.set_flags(vec![true]);
        let mut bytes: Vec<u8> = value.encode();
        let at: usize = ListKindsRef::new(&bytes)
            .unwrap()
            .flags()
            .as_bytes()
            .as_ptr() as usize
            - bytes.as_ptr() as usize;
        bytes[at] = flag;
        bytes
    };
    assert!(ListKinds::decode(&flags(0)).is_ok());
    assert_eq!(
        ListKinds::decode(&flags(2)),
        Err(DecodeError::InvalidBool { value: 2 })
    );
}

#[test]
fn messages_hold_lists() {
    let bytes: [u8; 20] = [
        2, 0, // count
        1, 0, 2, 0, // tags
        2, 0, 0, 0, // ends
        9, 8, // bytes
        7, 0, 0, 0, 6, 0, 0, 0, // numbers
    ];
    let value: WithLists = WithLists::default()
        .with_bytes(Bytes::from([9, 8]))
        .with_numbers(vec![7, 6]);
    assert_encoding!(WithLists, value, &bytes);

    let view: WithListsRef = WithListsRef::new(&bytes).unwrap();
    assert_eq!(view.bytes().unwrap(), [9, 8]);
    assert_eq!(view.numbers().unwrap().decode(), [7, 6]);
    assert!(view.texts().is_none());
}

#[test]
fn messages_tell_absent_lists_from_empty_lists() {
    let empty: WithLists = WithLists::default().with_texts(Vec::default());
    assert_encoding!(WithLists, empty.clone(), &[1, 0, 3, 0]);
    assert_eq!(empty.texts(), Some(&[][..]));
    assert_eq!(WithLists::default().texts(), None);
}

#[test]
fn messages_hold_lists_of_every_kind() {
    let child: WithLists = WithLists::default().with_texts(vec!["child".to_string()]);
    let value: WithLists = WithLists::default()
        .with_bytes(Bytes::from([1]))
        .with_texts(vec!["a".to_string(), "bc".to_string()])
        .with_points(vec![Point::new(1, 2), Point::new(3, 4)])
        .with_entries(vec![Entry::new(5, "five".to_string())])
        .with_children(vec![child.clone(), WithLists::default()]);
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(WithLists, value.clone(), &bytes);

    let view: WithListsRef = WithListsRef::new(&bytes).unwrap();
    assert_eq!(view.points().unwrap().get(1).unwrap().x(), 3);
    assert_eq!(view.entries().unwrap().get(0).unwrap().text(), "five");
    let children: ListRef<WithLists> = view.children().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children.get(0).unwrap().decode(), child);

    let json: String = serde_json::to_string(&value).unwrap();
    assert_eq!(serde_json::from_str::<WithLists>(&json).unwrap(), value);
}

#[test]
fn messages_reject_invalid_lists() {
    let bytes: [u8; 7] = [1, 0, 2, 0, 7, 0, 0];
    assert_eq!(WithLists::decode(&bytes), Err(DecodeError::InvalidList));
}

#[test]
fn variants_hold_lists() {
    assert_encoding!(
        ListCases,
        ListCases::Numbers(vec![7, 6]),
        &[2, 0, 7, 0, 0, 0, 6, 0, 0, 0]
    );
    assert_encoding!(ListCases, ListCases::Blob(Bytes::default()), &[1, 0]);
    assert_encoding!(
        ListCases,
        ListCases::Texts(vec!["hi".to_string()]),
        &[3, 0, 1, 0, 0, 0, b'h', b'i']
    );

    let value: ListCases = ListCases::Points(vec![Point::new(1, 2), Point::new(3, 4)]);
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(ListCases, value.clone(), &bytes);
    assert_eq!(
        value.as_points(),
        Some(&[Point::new(1, 2), Point::new(3, 4)][..])
    );
    assert_eq!(value.as_numbers(), None);

    let view: ListCasesRef = ListCasesRef::new(&bytes).unwrap();
    assert_eq!(view.as_points().unwrap().get(1).unwrap().y(), 4);
    match view.case() {
        ListCasesCase::Points(points) => assert_eq!(points.len(), 2),
        other => panic!("{:?}", other),
    }
}

#[test]
fn variants_reject_invalid_lists() {
    assert_eq!(
        ListCases::decode(&[2, 0, 7, 0, 0]),
        Err(DecodeError::InvalidList)
    );
    assert_eq!(
        ListCases::decode(&[3, 0, 1, 0, 0, 0, 0xFF]),
        Err(DecodeError::InvalidUtf8)
    );
}

#[test]
fn variant_lists_are_arrays_in_json() {
    assert_json(
        &ListCases::Numbers(vec![7, 6]),
        r#"{"var":"Numbers","val":[7,6]}"#,
    );
    assert_eq!(
        serde_json::from_str::<ListCases>(r#"{"val":["a","b"],"var":"Texts"}"#).unwrap(),
        ListCases::Texts(vec!["a".to_string(), "b".to_string()])
    );
}

#[test]
fn bytes_are_base_64_in_json() {
    assert_json(
        &WithLists::default().with_bytes(Bytes::from([9, 8])),
        r#"{"bytes":"CQg="}"#,
    );
    assert_json(
        &ListCases::Blob(Bytes::from([1, 2, 3])),
        r#"{"var":"Blob","val":"AQID"}"#,
    );
    assert_json(
        &ListCases::Blob(Bytes::default()),
        r#"{"var":"Blob","val":""}"#,
    );
    for json in [
        r#"{"bytes":[9,8]}"#,  // a list of numbers
        r#"{"bytes":"!!!!"}"#, // invalid base-64
        r#"{"bytes":9}"#,      // a number
    ] {
        assert!(serde_json::from_str::<WithLists>(json).is_err(), "{}", json);
    }
}

#[test]
fn bytes_read_as_slices() {
    let value: WithLists = WithLists::default().with_bytes(Bytes::from([9, 8]));
    assert_eq!(value.bytes(), Some(&[9_u8, 8][..]));
    let bytes: Bytes = vec![1, 2].into();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes.as_bytes(), [1, 2]);
    assert_eq!(Vec::from(bytes), [1, 2]);
}

#[test]
fn lists_are_arrays_in_json() {
    assert_json(
        &Numbers::new(7, vec![1, 2]),
        r#"{"count":7,"values":[1,2]}"#,
    );
    assert_json(&texts(), r#"{"texts":["hi","","yo!"]}"#);
    assert_json(
        &Nested::new(vec![Bytes::from([1, 2]), Bytes::default()]),
        r#"{"rows":["AQI=",""]}"#,
    );
    let value: ListKinds = list_kinds();
    let json: String = serde_json::to_string(&value).unwrap();
    assert_eq!(serde_json::from_str::<ListKinds>(&json).unwrap(), value);
}

/// Gets the sum of the count & the values of any form of the numbers.
fn sum<N: NumbersView>(numbers: &N) -> u32 {
    let values = numbers.values();
    u32::from(numbers.count()) + values.iter().sum::<u32>() + values.len() as u32
}

/// Gets the texts of any form, joined.
fn join<T: TextsView>(texts: &T) -> String {
    texts.texts().iter().collect::<Vec<&str>>().join(",")
}

/// Gets the length of each row of any form.
fn row_lens<N: NestedView>(nested: &N) -> Vec<usize> {
    nested.rows().iter().map(<[u8]>::len).collect()
}

/// Reads a list of each kind from any form.
fn digest<K: ListKindsView>(kinds: &K) -> String {
    let points: u32 = kinds
        .points()
        .iter()
        .map(|point| point.x() + point.y())
        .sum();
    let entries: Vec<String> = kinds
        .entries()
        .iter()
        .map(|entry| format!("{}{}", entry.id(), entry.text()))
        .collect();
    format!(
        "{:?} {} {:?} {} {} {:?} {:?} {:?}",
        kinds.bytes(),
        kinds.flags().len(),
        kinds.signed().get(0),
        points,
        entries.join("+"),
        kinds.texts().get(1),
        kinds.colors().iter().collect::<Vec<Color>>(),
        kinds.maybe().map(|maybe| maybe.iter().sum::<u16>()),
    )
}

#[test]
fn views_read_lists_of_every_form() {
    let value: Numbers = Numbers::new(7, vec![1, 2]);
    let mut bytes: Vec<u8> = value.encode();
    assert_eq!(sum(&value), 12);
    assert_eq!(sum(&&value), 12);
    assert_eq!(sum(&NumbersRef::new(&bytes).unwrap()), 12);
    assert_eq!(sum(&NumbersBuf::from(&value)), 12);
    assert_eq!(sum(&NumbersMut::new(&mut bytes).unwrap()), 12);

    assert_eq!(join(&texts()), "hi,,yo!");
    assert_eq!(join(&TextsRef::new(&texts_bytes()).unwrap()), "hi,,yo!");
}

#[test]
fn views_read_nested_lists() {
    let value: Nested = Nested::new(vec![
        Bytes::from([1, 2]),
        Bytes::default(),
        Bytes::from([3]),
    ]);
    let bytes: Vec<u8> = value.encode();
    assert_eq!(row_lens(&value), [2, 0, 1]);
    assert_eq!(row_lens(&NestedRef::new(&bytes).unwrap()), [2, 0, 1]);
}

#[test]
fn views_read_lists_of_every_kind() {
    let value: ListKinds = list_kinds();
    let mut bytes: Vec<u8> = value.encode();
    let expected: String = digest(&value);
    assert_eq!(
        expected,
        "[1, 2, 3] 2 Some(-1) 10 5five+6 Some(\"bc\") [Red] Some(23)"
    );
    assert_eq!(digest(&ListKindsRef::new(&bytes).unwrap()), expected);
    assert_eq!(digest(&ListKindsBuf::from(&value)), expected);
    assert_eq!(digest(&ListKindsMut::new(&mut bytes).unwrap()), expected);
}

/// Gets the count of the numbers of any form of the cases.
fn numbers_len<C: ListCasesView>(cases: &C) -> Option<usize> {
    cases.as_numbers().map(|numbers| numbers.len())
}

/// Gets the count of the lists of any form of the message.
fn list_lens<W: WithListsView>(message: &W) -> (Option<usize>, Option<usize>) {
    (
        message.bytes().map(<[u8]>::len),
        message.children().map(|children| children.len()),
    )
}

#[test]
fn views_read_the_lists_of_variants_and_messages() {
    let value: ListCases = ListCases::Numbers(vec![7, 6]);
    assert_eq!(numbers_len(&value), Some(2));
    assert_eq!(numbers_len(&ListCasesBuf::from(&value)), Some(2));
    assert_eq!(numbers_len(&ListCases::Number(1)), None);

    let message: WithLists = WithLists::default()
        .with_bytes(Bytes::from([1]))
        .with_children(vec![WithLists::default(), WithLists::default()]);
    let bytes: Vec<u8> = message.encode();
    assert_eq!(list_lens(&message), (Some(1), Some(2)));
    assert_eq!(
        list_lens(&WithListsRef::new(&bytes).unwrap()),
        (Some(1), Some(2))
    );
}

#[test]
fn encoded_lists_are_equal_by_their_bytes() {
    let bytes: Vec<u8> = texts_bytes();
    let (texts, same): (TextsRef, TextsRef) = (
        TextsRef::new(&bytes).unwrap(),
        TextsRef::new(&bytes).unwrap(),
    );
    assert_eq!(texts.texts(), same.texts());
    let other: Vec<u8> = Texts::new(vec!["hi".to_string()]).encode();
    assert_ne!(texts.texts(), TextsRef::new(&other).unwrap().texts());
}

/// Gets the `count` of messages, each in the children of the one before it, around the `inner`.
fn nested_children(count: usize, inner: WithLists) -> WithLists {
    (1..count).fold(inner, |child, _| {
        WithLists::default().with_children(vec![child])
    })
}

#[test]
fn lists_are_not_levels() {
    let points: WithLists = WithLists::default().with_points(vec![Point::new(1, 2)]);
    let entries: WithLists =
        WithLists::default().with_entries(vec![Entry::new(1, "a".to_string())]);
    // a writer that is given one level more than a value can nest
    let encode_deep = |value: WithLists| -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::default();
        value.encode_to_with_depth(&mut bytes, 65);
        bytes
    };

    // the points are fixed-size, so the deepest message can hold them
    let deepest: WithLists = nested_children(64, points.clone());
    assert_eq!(WithLists::decode(&deepest.encode()), Ok(deepest));
    assert_eq!(
        WithLists::decode(&encode_deep(nested_children(65, points))),
        Err(DecodeError::TooDeep)
    );

    // an entry has a variable tail, so it is a level of its own
    let deepest: WithLists = nested_children(63, entries.clone());
    assert_eq!(WithLists::decode(&deepest.encode()), Ok(deepest));
    assert_eq!(
        WithLists::decode(&encode_deep(nested_children(64, entries))),
        Err(DecodeError::TooDeep)
    );
}

#[test]
#[should_panic(expected = "the value nests deeper than 64 levels")]
fn writers_refuse_list_elements_that_nest_too_deep() {
    let entries: WithLists =
        WithLists::default().with_entries(vec![Entry::new(1, "a".to_string())]);
    let _ = nested_children(64, entries).encode();
}
