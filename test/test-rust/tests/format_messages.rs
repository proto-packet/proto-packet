mod common;

use common::ID;
use proto_packet::io::DecodeError;
use proto_packet::types::Float64;
use proto_packet_test::format::examples::{Color, Entry, Point};
use proto_packet_test::format::messages::{
    AllKinds, AllKindsMut, AllKindsRef, Item, ItemBuf, ItemMut, ItemRef, ItemView, LoopMessage,
    LoopStruct, Newer, NewerRef, Older, OlderRef, WithMessages, WithMessagesMut, WithMessagesRef,
};
use proto_packet_test::messages::boxed::{Comment, CommentRef, CommentView, CommentViewMut};
use proto_packet_test::messages::empty::{EmptyMessage, EmptyMessageRef};
use proto_packet_test::messages::tags::{EdgeTags, UnorderedTags};

/// Asserts the `value` of the `type` encodes to the `bytes`, & decodes back from them.
macro_rules! assert_encoding {
    ($type:ty, $value:expr, $bytes:expr) => {{
        let value: $type = $value;
        let bytes: &[u8] = $bytes;
        assert_eq!(value.encode(), bytes);
        assert_eq!(value.encoded_len(), bytes.len());
        assert_eq!(<$type>::decode(bytes), Ok(value.clone()));
        assert_eq!(unsafe { <$type>::decode_unchecked(bytes) }, value);
        let (len, encoded): (usize, Vec<u8>) = common::ViewEncode::view_encode(&value);
        assert_eq!(encoded, bytes);
        assert_eq!(len, bytes.len());
        common::assert_one_encoding(bytes, |changed| {
            let value: $type = <$type>::decode(changed).ok()?;
            assert_eq!(value.encoded_len(), changed.len());
            Some(value.encode())
        });
    }};
}

common::plain!(
    proto_packet_test::format::messages::AllKindsView,
    AllKinds,
    {
        fn flag(&self) -> Option<bool>;
        fn byte(&self) -> Option<u8>;
        fn large(&self) -> Option<u128>;
        fn id(&self) -> Option<proto_packet::types::Uuid>;
        fn color(&self) -> Option<proto_packet_test::format::examples::Color>;
        fn text(&self) -> Option<&str>;
        fn point(&self) -> Option<impl proto_packet_test::format::examples::PointView>;
        fn entry(&self) -> Option<impl proto_packet_test::format::examples::EntryView>;
        fn item(&self) -> Option<impl proto_packet_test::format::messages::ItemView>;
        fn boxed_color(&self) -> Option<proto_packet_test::format::examples::Color>;
        fn signed(&self) -> Option<i64>;
        fn float(&self) -> Option<proto_packet::types::Float64>;
        fn at(&self) -> Option<proto_packet::types::Timestamp>;
        fn price(&self) -> Option<proto_packet::types::Decimal<2>>;
    }
);

common::plain!(proto_packet_test::messages::boxed::CommentView, Comment, {
    fn text(&self) -> Option<&str>;
    fn reply(&self) -> Option<impl proto_packet_test::messages::boxed::CommentView>;
});

common::plain!(proto_packet_test::messages::tags::EdgeTagsView, EdgeTags, {
    fn min(&self) -> Option<u32>;
    fn max(&self) -> Option<u32>;
});

common::plain!(
    proto_packet_test::messages::empty::EmptyMessageView,
    EmptyMessage,
    {}
);

common::plain!(proto_packet_test::format::messages::ItemView, Item, {
    fn text(&self) -> Option<&str>;
    fn id(&self) -> Option<u32>;
});

common::plain!(
    proto_packet_test::format::messages::LoopMessageView,
    LoopMessage,
    {
        fn value(&self) -> Option<u32>;
        fn next(&self) -> Option<impl proto_packet_test::format::messages::LoopStructView>;
    }
);

common::plain!(
    proto_packet_test::messages::tags::UnorderedTagsView,
    UnorderedTags,
    {
        fn three(&self) -> Option<u32>;
        fn one(&self) -> Option<u32>;
        fn two(&self) -> Option<u32>;
    }
);

common::plain!(
    proto_packet_test::format::messages::WithMessagesView,
    WithMessages,
    {
        fn before(&self) -> &str;
        fn item(&self) -> impl proto_packet_test::format::messages::ItemView;
        fn number(&self) -> u64;
        fn maybe(&self) -> Option<impl proto_packet_test::format::messages::ItemView>;
    }
);

/// Gets the value of the `Item` example of `format/binary.md`.
fn item() -> Item {
    Item::default().with_text("hi".to_string()).with_id(7)
}

/// Gets the bytes of the `Item` example of `format/binary.md`.
fn item_bytes() -> Vec<u8> {
    vec![
        2, 0, // count
        1, 0, 5, 0, // tags
        2, 0, 0, 0, // ends
        b'h', b'i', // text
        7, 0, 0, 0, // id
    ]
}

/// Gets a message with a field of each kind.
fn all_kinds() -> AllKinds {
    AllKinds::default()
        .with_flag(true)
        .with_byte(1)
        .with_large(2)
        .with_id(ID)
        .with_color(Color::Red)
        .with_text("text".to_string())
        .with_point(Point::new(3, 4))
        .with_entry(Entry::new(5, "entry".to_string()))
        .with_item(item())
        .with_boxed_color(Color::Red)
        .with_signed(-2)
        .with_float(Float64::new(1.5))
}

/// Gets a newer message, with a field around each of the older fields.
fn newer() -> Newer {
    Newer::default()
        .with_id(1)
        .with_name("Ada".to_string())
        .with_nickname("A".to_string())
        .with_age(36)
        .with_entry(Entry::new(2, "entry".to_string()))
}

#[test]
fn item_example() {
    let bytes: Vec<u8> = item_bytes();
    assert_eq!(bytes.len(), 16);
    assert_encoding!(Item, item(), &bytes);

    let value: ItemRef = ItemRef::new(&bytes).unwrap();
    assert_eq!((value.text(), value.id()), (Some("hi"), Some(7)));
}

#[test]
fn absent_fields_are_not_listed() {
    let bytes: [u8; 8] = [1, 0, 5, 0, 7, 0, 0, 0];
    assert_encoding!(Item, Item::default().with_id(7), &bytes);

    let value: ItemRef = ItemRef::new(&bytes).unwrap();
    assert_eq!((value.text(), value.id()), (None, Some(7)));
}

#[test]
fn empty_messages() {
    assert_encoding!(Item, Item::default(), &[0, 0]);
    assert_encoding!(EmptyMessage, EmptyMessage::default(), &[0, 0]);
    EmptyMessageRef::new(&[0, 0]).unwrap();
    assert_eq!(
        Item::decode(&[0, 0, 0xFF]),
        Err(DecodeError::TooLong {
            expected: 2,
            actual: 3,
        })
    );
}

#[test]
fn every_kind_round_trips() {
    let value: AllKinds = all_kinds();
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(AllKinds, value.clone(), &bytes);

    let kinds: AllKindsRef = AllKindsRef::new(&bytes).unwrap();
    assert_eq!(kinds.flag(), Some(true));
    assert_eq!(kinds.large(), Some(2));
    assert_eq!(kinds.id(), Some(ID));
    assert_eq!(kinds.color(), Some(Color::Red));
    assert_eq!(kinds.point().unwrap().y(), 4);
    assert_eq!(kinds.entry().unwrap().text(), "entry");
    assert_eq!(kinds.item().unwrap().id(), Some(7));
    assert_eq!(kinds.boxed_color(), Some(Color::Red));
    assert_eq!(kinds.signed(), Some(-2));
    assert_eq!(kinds.float(), Some(Float64::new(1.5)));
}

#[test]
fn floats_are_their_bits() {
    let bytes: [u8; 12] = [1, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0xC0];
    let value: AllKinds = AllKinds::default().with_float(Float64::new(-2.0));
    assert_encoding!(AllKinds, value, &bytes);

    let mut other_nan: [u8; 12] = bytes;
    other_nan[10..].copy_from_slice(&[0xF8, 0xFF]);
    assert_eq!(AllKinds::decode(&other_nan), Err(DecodeError::InvalidFloat));
}

#[test]
fn signed_integers_are_twos_complement() {
    let bytes: [u8; 12] = [1, 0, 11, 0, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    assert_encoding!(AllKinds, AllKinds::default().with_signed(-2), &bytes);
}

#[test]
fn tags_are_sorted() {
    let value: UnorderedTags = UnorderedTags::default()
        .with_three(3)
        .with_one(1)
        .with_two(2);
    let bytes: Vec<u8> = value.encode();
    assert_eq!(bytes[..8], [3, 0, 1, 0, 2, 0, 3, 0]);
    assert_encoding!(UnorderedTags, value, &bytes);
}

#[test]
fn the_size_ignores_the_tag_numbers() {
    let value: EdgeTags = EdgeTags::default().with_min(1).with_max(2);
    let bytes: Vec<u8> = value.encode();
    assert_eq!(bytes.len(), 2 + 4 + 4 + 8);
    assert_encoding!(EdgeTags, value, &bytes);
}

#[test]
fn encodings_append() {
    let mut out: Vec<u8> = vec![0xAA];
    item().encode_to(&mut out);
    assert_eq!(out[0], 0xAA);
    assert_eq!(out[1..], item_bytes());
}

#[test]
fn older_readers_skip_newer_fields() {
    let bytes: Vec<u8> = newer().encode();
    let older: OlderRef = OlderRef::new(&bytes).unwrap();
    assert_eq!((older.name(), older.age()), (Some("Ada"), Some(36)));
    assert_eq!(older.as_bytes(), bytes);
}

#[test]
fn unknown_fields_survive_decoding() {
    let bytes: Vec<u8> = newer().encode();
    let older: Older = Older::decode(&bytes).unwrap();
    let tags: Vec<u16> = older
        .unknown_fields()
        .iter()
        .map(|field| field.tag_number().value())
        .collect();
    assert_eq!(tags, [1, 3, 5]);

    assert_eq!(older.encode(), bytes);
    assert_eq!(older.encoded_len(), bytes.len());
    assert_eq!(Newer::decode(&older.encode()), Ok(newer()));
}

#[test]
fn unknown_fields_survive_edits() {
    let mut older: Older = Older::decode(&newer().encode()).unwrap();
    older.set_age(40);
    let edited: Newer = Newer::decode(&older.encode()).unwrap();
    assert_eq!(edited, newer().with_age(40));
}

#[test]
fn newer_readers_see_older_fields_as_absent() {
    let older: Older = Older::default().with_name("Ada".to_string()).with_age(36);
    let bytes: Vec<u8> = older.encode();
    let value: NewerRef = NewerRef::new(&bytes).unwrap();
    assert_eq!(value.name(), Some("Ada"));
    assert_eq!(value.nickname(), None);
    assert!(value.entry().is_none());
}

#[test]
fn rejects_extra_bytes() {
    let mut bytes: Vec<u8> = item_bytes();
    bytes.push(0xFF);
    assert_eq!(
        Item::decode(&bytes),
        Err(DecodeError::TooLong {
            expected: 4,
            actual: 5,
        })
    );
}

#[test]
fn strings_are_their_whole_value() {
    let bytes: [u8; 7] = [1, 0, 1, 0, b'h', b'i', b'!'];
    assert_eq!(ItemRef::new(&bytes).unwrap().text(), Some("hi!"));
}

#[test]
fn rejects_bool_bytes_above_1() {
    let mut bytes: Vec<u8> = AllKinds::default().with_flag(true).encode();
    *bytes.last_mut().unwrap() = 2;
    assert_eq!(
        AllKinds::decode(&bytes),
        Err(DecodeError::InvalidBool { value: 2 })
    );
}

#[test]
fn rejects_short_bytes() {
    assert_eq!(
        Item::decode(&[]),
        Err(DecodeError::TooShort {
            expected: 2,
            actual: 0,
        })
    );
    assert_eq!(
        Item::decode(&[1, 0]),
        Err(DecodeError::TooShort {
            expected: 4,
            actual: 2,
        })
    );
}

#[test]
fn rejects_invalid_tags() {
    let with_tags = |first: [u8; 2], second: [u8; 2]| -> Vec<u8> {
        let mut bytes: Vec<u8> = item_bytes();
        bytes[2..4].copy_from_slice(&first);
        bytes[4..6].copy_from_slice(&second);
        bytes
    };
    for (first, second) in [
        ([5, 0], [1, 0]), // decreasing
        ([5, 0], [5, 0]), // duplicate
        ([0, 0], [5, 0]), // zero
    ] {
        assert_eq!(
            Item::decode(&with_tags(first, second)),
            Err(DecodeError::InvalidTags)
        );
    }
}

#[test]
fn rejects_end_tables() {
    let mut too_long: Vec<u8> = item_bytes();
    too_long[6] = 7;
    assert_eq!(Item::decode(&too_long), Err(DecodeError::InvalidEndTable));

    let decreasing: [u8; 23] = [
        3, 0, // count
        1, 0, 5, 0, 7, 0, // tags
        2, 0, 0, 0, 1, 0, 0, 0, // ends
        b'h', b'i', // text
        7, 0, 0, 0, // id
        9, // unknown
    ];
    assert_eq!(Item::decode(&decreasing), Err(DecodeError::InvalidEndTable));
}

#[test]
fn rejects_short_values() {
    let bytes: [u8; 6] = [1, 0, 5, 0, 7, 0];
    assert_eq!(
        Item::decode(&bytes),
        Err(DecodeError::TooShort {
            expected: 4,
            actual: 2,
        })
    );
}

#[test]
fn rejects_invalid_utf8() {
    let mut bytes: Vec<u8> = item_bytes();
    bytes[10] = 0xFF;
    assert_eq!(Item::decode(&bytes), Err(DecodeError::InvalidUtf8));
}

#[test]
fn rejects_invalid_enums() {
    let mut bytes: Vec<u8> = AllKinds::default().with_color(Color::Red).encode();
    let last: usize = bytes.len();
    bytes[last - 2] = 0;
    assert_eq!(
        AllKinds::decode(&bytes),
        Err(DecodeError::InvalidTagNumber { value: 0 })
    );
}

#[test]
fn rejects_invalid_nested_values() {
    let mut bytes: Vec<u8> = AllKinds::default().with_item(item()).encode();
    let last: usize = bytes.len();
    bytes[last - 6] = 0xFF; // the text of the item
    assert_eq!(AllKinds::decode(&bytes), Err(DecodeError::InvalidUtf8));
}

#[test]
fn unknown_fields_are_not_validated() {
    let mut bytes: Vec<u8> = Newer::default().with_nickname("A".to_string()).encode();
    *bytes.last_mut().unwrap() = 0xFF;
    Older::decode(&bytes).unwrap();
    assert!(Newer::decode(&bytes).is_err());
}

#[test]
fn mut_reaches_present_nested_values() {
    let mut bytes: Vec<u8> = all_kinds().encode();
    let mut kinds: AllKindsMut = AllKindsMut::new(&mut bytes).unwrap();
    kinds.point_mut().unwrap().set_y(9);
    kinds.entry_mut().unwrap().set_id(8);
    assert!(kinds.item_mut().is_some());
    let decoded: AllKinds = AllKinds::decode(&bytes).unwrap();
    assert_eq!(decoded.point().unwrap().y(), 9);
    assert_eq!(decoded.entry().unwrap().id(), 8);

    let mut bytes: Vec<u8> = AllKinds::default().encode();
    let mut kinds: AllKindsMut = AllKindsMut::new(&mut bytes).unwrap();
    assert!(kinds.item_mut().is_none());
}

#[test]
fn bufs_own_the_encoding() {
    let buf: ItemBuf = ItemBuf::from(&item());
    assert_eq!(buf.as_bytes(), item_bytes());
    assert_eq!(buf.as_ref().id(), Some(7));
    assert_eq!(buf.decode(), item());
    assert_eq!(
        ItemBuf::new(item_bytes()).unwrap().into_bytes(),
        item_bytes()
    );
    assert_eq!(
        ItemRef::new(&item_bytes()).unwrap().to_buf().decode(),
        item()
    );
}

/// Gets the fields of any form of an item.
fn item_fields<I: ItemView>(item: &I) -> (Option<String>, Option<u32>) {
    (item.text().map(str::to_string), item.id())
}

#[test]
fn views_read_every_form() {
    let expected = (Some("hi".to_string()), Some(7));
    let mut bytes: Vec<u8> = item_bytes();
    assert_eq!(item_fields(&item()), expected);
    assert_eq!(item_fields(&&item()), expected);
    assert_eq!(item_fields(&ItemRef::new(&bytes).unwrap()), expected);
    assert_eq!(item_fields(&ItemBuf::from(&item())), expected);
    assert_eq!(item_fields(&ItemMut::new(&mut bytes).unwrap()), expected);
}

/// Gets the depth of any form of a comment.
fn depth<C: CommentView>(comment: &C) -> usize {
    1 + comment.reply().map_or(0, |reply| depth(&reply))
}

/// Gets the depth of any editable form of a comment, through its mutable replies.
fn depth_mut<C: CommentViewMut>(comment: &mut C) -> usize {
    1 + comment
        .reply_mut()
        .map_or(0, |mut reply| depth_mut(&mut reply))
}

#[test]
fn recursive_messages() {
    let tail: Comment = Comment::default().with_text("c".to_string());
    let middle: Comment = Comment::default()
        .with_text("b".to_string())
        .with_reply(tail);
    let value: Comment = Comment::default()
        .with_text("a".to_string())
        .with_reply(middle);
    let mut bytes: Vec<u8> = value.encode();
    assert_encoding!(Comment, value.clone(), &bytes);
    assert_eq!(depth(&value), 3);
    assert_eq!(depth(&CommentRef::new(&bytes).unwrap()), 3);

    let mut edit = value.clone();
    assert_eq!(depth_mut(&mut edit), 3);
    let mut comment = proto_packet_test::messages::boxed::CommentMut::new(&mut bytes).unwrap();
    assert_eq!(depth_mut(&mut comment), 3);
}

/// Gets a struct with messages.
fn with_messages() -> WithMessages {
    WithMessages::new(
        "before".to_string(),
        item(),
        8,
        Some(Item::default().with_id(9)),
    )
}

#[test]
fn structs_hold_messages() {
    let value: WithMessages = with_messages();
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(WithMessages, value, &bytes);

    let view: WithMessagesRef = WithMessagesRef::new(&bytes).unwrap();
    assert_eq!(view.before(), "before");
    assert_eq!(view.item().id(), Some(7));
    assert_eq!(view.maybe().unwrap().id(), Some(9));

    let absent: WithMessages = WithMessages::new("x".to_string(), item(), 8, None);
    let bytes: Vec<u8> = absent.encode();
    assert_encoding!(WithMessages, absent, &bytes);
    assert!(WithMessagesRef::new(&bytes).unwrap().maybe().is_none());
}

#[test]
fn structs_reject_invalid_messages() {
    let mut bytes: Vec<u8> = with_messages().encode();
    let at: usize = bytes.windows(2).position(|window| window == b"hi").unwrap();
    bytes[at] = 0xFF;
    assert_eq!(WithMessages::decode(&bytes), Err(DecodeError::InvalidUtf8));
}

#[test]
fn structs_reach_their_messages_in_place() {
    let mut bytes: Vec<u8> = with_messages().encode();
    let mut value: WithMessagesMut = WithMessagesMut::new(&mut bytes).unwrap();
    assert_eq!(value.item_mut().as_ref().id(), Some(7));
    assert_eq!(value.maybe_mut().unwrap().as_ref().id(), Some(9));
    assert_eq!(value.as_ref().before(), "before");
    WithMessagesRef::new(value.as_bytes()).unwrap();
}

#[test]
fn messages_hold_loops_of_structs() {
    let inner: LoopMessage = LoopMessage::default().with_value(3);
    let middle: LoopStruct = LoopStruct::new(2, Some(inner));
    let value: LoopMessage = LoopMessage::default().with_value(1).with_next(middle);
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(LoopMessage, value, &bytes);
}

/// Gets an item with an unknown field, a `u8` of 9 with the tag number 7.
fn item_with_unknown() -> Item {
    let bytes: [u8; 13] = [
        2, 0, // count
        1, 0, 7, 0, // tags
        2, 0, 0, 0, // ends
        b'h', b'i', // text
        9,    // unknown
    ];
    Item::decode(&bytes).unwrap()
}

#[test]
fn known_fields_stay_plain_json() {
    assert_eq!(
        serde_json::to_string(&item()).unwrap(),
        r#"{"text":"hi","id":7}"#
    );
    assert_eq!(serde_json::to_string(&Item::default()).unwrap(), "{}");
    assert_eq!(
        serde_json::from_str::<Item>(r#"{"id":7,"text":"hi"}"#).unwrap(),
        item()
    );
}

#[test]
fn json_ignores_nulls_and_other_keys() {
    assert_eq!(
        serde_json::from_str::<Item>(r#"{"text":null,"other":{"a":[1]},"id":7}"#).unwrap(),
        Item::default().with_id(7)
    );
}

#[test]
fn large_integers_round_trip_through_json() {
    let value: AllKinds = AllKinds::default().with_large(u128::MAX);
    let json: String = serde_json::to_string(&value).unwrap();
    assert_eq!(serde_json::from_str::<AllKinds>(&json).unwrap(), value);
}

#[test]
fn unknown_fields_are_base_64_json() {
    assert_eq!(
        serde_json::to_string(&item_with_unknown()).unwrap(),
        r#"{"text":"hi","7":"CQ=="}"#
    );

    let value: Newer = Newer::default()
        .with_id(1)
        .with_name("Ada".to_string())
        .with_nickname("A".to_string())
        .with_age(36);
    let older: Older = Older::decode(&value.encode()).unwrap();
    assert_eq!(
        serde_json::to_string(&older).unwrap(),
        r#"{"name":"Ada","age":36,"1":"AQAAAAAAAAA=","3":"QQ=="}"#
    );
}

#[test]
fn unknown_fields_round_trip_through_json() {
    let bytes: Vec<u8> = newer().encode();
    let older: Older = Older::decode(&bytes).unwrap();
    let json: String = serde_json::to_string(&older).unwrap();
    let reread: Older = serde_json::from_str(&json).unwrap();
    assert_eq!(reread, older);
    assert_eq!(reread.encode(), bytes);
}

#[test]
fn known_fields_decode_from_base_64() {
    let older: Older = Older::decode(&newer().encode()).unwrap();
    let json: String = serde_json::to_string(&older).unwrap();
    assert_eq!(serde_json::from_str::<Newer>(&json).unwrap(), newer());
}

#[test]
fn unknown_fields_can_come_first() {
    assert_eq!(
        serde_json::from_str::<Item>(r#"{"7":"CQ==","text":"hi"}"#).unwrap(),
        item_with_unknown()
    );
}

#[test]
fn empty_unknown_fields_round_trip_through_json() {
    let value: Item = serde_json::from_str(r#"{"7":""}"#).unwrap();
    assert_eq!(value.unknown_fields()[0].bytes(), [0_u8; 0]);
    assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"7":""}"#);
}

#[test]
fn nested_unknown_fields_round_trip_through_json() {
    let inner: Item = item_with_unknown();

    let message: AllKinds = AllKinds::default().with_item(inner.clone());
    let json: String = serde_json::to_string(&message).unwrap();
    assert_eq!(json, r#"{"item":{"text":"hi","7":"CQ=="}}"#);
    assert_eq!(serde_json::from_str::<AllKinds>(&json).unwrap(), message);

    let structure: WithMessages = WithMessages::new("x".to_string(), inner, 8, None);
    let json: String = serde_json::to_string(&structure).unwrap();
    assert_eq!(
        serde_json::from_str::<WithMessages>(&json).unwrap(),
        structure
    );
}

#[test]
fn json_rejects_invalid_messages() {
    for json in [
        r#"{"0":""}"#,                 // an invalid tag number
        r#"{"65536":""}"#,             // a tag number above 65,535
        r#"{"7":"!!!!"}"#,             // invalid base-64
        r#"{"7":5}"#,                  // a value that isn't base-64 text
        r#"{"5":"AQ=="}"#,             // a known field too short for its type
        r#"{"1":"/w=="}"#,             // a known field that isn't valid UTF-8
        r#"{"7":"","7":""}"#,          // a duplicate tag number
        r#"{"text":"a","text":"b"}"#,  // a duplicate name
        r#"{"text":null,"text":"a"}"#, // a duplicate name after a null
        r#"{"0007":"CQ=="}"#,          // a tag number with leading zeros
        r#"{"text":"a","1":"aGk="}"#,  // a field by name & by tag number
    ] {
        assert!(serde_json::from_str::<Item>(json).is_err(), "{}", json);
    }
}

#[test]
fn older_readers_have_one_encoding_of_newer_bytes() {
    common::assert_one_encoding(&newer().encode(), |changed| {
        let older: Older = Older::decode(changed).ok()?;
        assert_eq!(older.encoded_len(), changed.len());
        Some(older.encode())
    });
}

#[test]
fn forms_are_equal_across_forms() {
    let bytes: Vec<u8> = newer().encode();
    let newer_ref: NewerRef = NewerRef::new(&bytes).unwrap();
    assert!(newer() == newer_ref && newer_ref == newer());
    assert!(newer().with_age(37) != newer_ref);
    assert_eq!(newer_ref, NewerRef::new(&bytes.clone()).unwrap());

    // an older reader keeps the fields it doesn't know, so its forms are still equal
    let older: Older = Older::decode(&bytes).unwrap();
    let older_ref: OlderRef = OlderRef::new(&bytes).unwrap();
    assert!(older == older_ref && older_ref == older);
    assert!(older.clone().with_age(37) != older_ref);

    let item: Item = Item::default().with_id(7);
    let buf: ItemBuf = ItemBuf::from(&item);
    assert!(item == buf && buf == item && buf == buf.as_ref() && buf.as_ref() == buf);
    assert!(Item::default() != buf);
}

/// Gets the `count` of comments, each the reply of the one before it.
fn comments(count: usize) -> Comment {
    (1..count).fold(Comment::default().with_text("x".to_string()), |reply, _| {
        Comment::default().with_reply(reply)
    })
}

#[test]
fn messages_nest_at_most_64_levels() {
    assert_eq!(Comment::decode(&comments(64).encode()), Ok(comments(64)));

    // a writer that is given one level more than a value can nest
    let mut deep: Vec<u8> = Vec::default();
    comments(65).encode_to_with_depth(&mut deep, 65);
    assert_eq!(Comment::decode(&deep), Err(DecodeError::TooDeep));
    assert_eq!(CommentRef::new(&deep), Err(DecodeError::TooDeep));
}

#[test]
#[should_panic(expected = "the value nests deeper than 64 levels")]
fn writers_refuse_messages_that_nest_too_deep() {
    let _ = comments(65).encode();
}
