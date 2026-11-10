mod common;

use common::assert_json;
use proto_packet::io::{DecodeError, MapRef, MapView};
use proto_packet::types::{Bytes, Float64};
use proto_packet_test::format::examples::{Color, Entry, EntryView, Point, PointView};
use proto_packet_test::format::maps::{
    MapCases, MapCasesRef, MapCasesView, MapKinds, MapKindsBuf, MapKindsRef, MapKindsView,
    NestedKeys, NestedKeysRef, NewerKey, NewerKeyRef, NewerKeys, NewerKeysRef, OlderKeys, Scores,
    ScoresBuf, ScoresMut, ScoresRef, ScoresView, Tagged, WithMaps, WithMapsRef,
};
use proto_packet_test::format::variants::NewerCases;
use std::collections::BTreeMap;

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

/// Gets the scores of the `Scores` example of `format/binary.md`.
fn scores() -> BTreeMap<String, u32> {
    BTreeMap::from([("a".to_string(), 1), ("bc".to_string(), 2)])
}

/// Gets the bytes of the `Scores` example of `format/binary.md`.
fn scores_bytes() -> Vec<u8> {
    [
        &[15, 0, 0, 0][..],        // the end of the keys
        &[2, 0, 0, 0, 9, 0, 0, 0], // the count & the ends of the keys
        b"a",
        b"bc",
        &[0],                      // padding
        &[1, 0, 0, 0, 2, 0, 0, 0], // the values
    ]
    .concat()
}

/// Gets a struct with a map of each kind.
fn map_kinds() -> MapKinds {
    MapKinds::new(
        BTreeMap::from([(1, true), (7, false)]),
        BTreeMap::from([(Color::Red, "red".to_string())]),
        BTreeMap::from([
            (Point::new(1, 2), Entry::new(3, "three".to_string())),
            (Point::new(1, 3), Entry::new(4, String::default())),
        ]),
        BTreeMap::from([
            ("a".to_string(), Bytes::from([1, 2])),
            ("b".to_string(), Bytes::default()),
        ]),
        BTreeMap::from([(1, BTreeMap::from([(2, 3)])), (4, BTreeMap::default())]),
        Some(BTreeMap::from([(-1, Float64::new(1.5)), (5, Float64::NAN)])),
    )
}

#[test]
fn scores_example() {
    let bytes: Vec<u8> = scores_bytes();
    assert_eq!(bytes.len(), 24);
    assert_encoding!(Scores, Scores::new(scores()), &bytes);

    let view: ScoresRef = ScoresRef::new(&bytes).unwrap();
    let map: MapRef<String, u32> = view.scores();
    assert_eq!(map.len(), 2);
    assert_eq!(
        (map.get("a"), map.get("bc"), map.get("b")),
        (Some(1), Some(2), None)
    );
    assert_eq!(map.keys().iter().collect::<Vec<&str>>(), ["a", "bc"]);
    assert_eq!(map.values().iter().collect::<Vec<u32>>(), [1, 2]);
    assert_eq!(
        map.iter().collect::<Vec<(&str, u32)>>(),
        [("a", 1), ("bc", 2)]
    );
}

#[test]
fn empty_maps_are_no_bytes() {
    assert_encoding!(Scores, Scores::new(BTreeMap::default()), &[]);
    let view: ScoresRef = ScoresRef::new(&[]).unwrap();
    assert!(view.scores().is_empty());
    assert_eq!(view.scores().len(), 0);
    assert_eq!(view.scores().get("a"), None);
}

#[test]
fn every_kind_round_trips() {
    let value: MapKinds = map_kinds();
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(MapKinds, value.clone(), &bytes);

    let kinds: MapKindsRef = MapKindsRef::new(&bytes).unwrap();
    assert_eq!(kinds.by_number().get(7), Some(false));
    assert_eq!(kinds.by_color().get(Color::Red), Some("red"));
    assert_eq!(kinds.by_point().len(), 2);
    assert_eq!(kinds.lists().get("a").unwrap(), [1, 2]);
    assert_eq!(kinds.maps().get(1).unwrap().get(2), Some(3));
    assert!(kinds.maps().get(4).unwrap().is_empty());
    assert_eq!(kinds.maybe().unwrap().get(5), Some(Float64::NAN));

    assert_eq!(value.by_number().get(&1), Some(&true));
    assert_eq!(value.maybe().map(BTreeMap::len), Some(2));
}

#[test]
fn absent_maps_are_not_empty_maps() {
    let absent: MapKinds = map_kinds().with_maybe(None);
    let empty: MapKinds = map_kinds().with_maybe(BTreeMap::default());
    assert_ne!(absent.encode(), empty.encode());
    assert_eq!(MapKinds::decode(&absent.encode()).unwrap().maybe(), None);
    assert_eq!(
        MapKinds::decode(&empty.encode()).unwrap().maybe(),
        Some(&BTreeMap::default())
    );
}

#[test]
fn rejects_keys_that_are_not_strictly_increasing() {
    let with_keys = |keys: &[u8; 3], end: u8| -> Vec<u8> {
        let mut bytes: Vec<u8> = scores_bytes();
        bytes[8] = end;
        bytes[12..15].copy_from_slice(keys);
        bytes
    };
    assert_eq!(
        Scores::decode(&with_keys(b"abc", 9)),
        Ok(Scores::new(scores()))
    );
    assert_eq!(
        Scores::decode(&with_keys(b"bca", 10)),
        Err(DecodeError::InvalidMap)
    );
    assert_eq!(Scores::decode(&with_keys(b"aax", 9)).map(|_| ()), Ok(()));

    let duplicate: Vec<u8> = [
        &[14, 0, 0, 0][..],
        &[2, 0, 0, 0, 9, 0, 0, 0],
        b"a",
        b"a",
        &[0, 0],
        &[1, 0, 0, 0, 2, 0, 0, 0],
    ]
    .concat();
    assert_eq!(Scores::decode(&duplicate), Err(DecodeError::InvalidMap));
}

#[test]
fn rejects_keys_and_values_that_do_not_match() {
    let mut more_values: Vec<u8> = scores_bytes();
    more_values.extend_from_slice(&[3, 0, 0, 0]);
    assert_eq!(Scores::decode(&more_values), Err(DecodeError::InvalidMap));

    let no_values: &[u8] = &scores_bytes()[..15];
    assert_eq!(Scores::decode(no_values), Err(DecodeError::InvalidMap));

    let mut keys_past_the_end: Vec<u8> = scores_bytes();
    keys_past_the_end[0] = 25;
    assert_eq!(
        Scores::decode(&keys_past_the_end),
        Err(DecodeError::InvalidMap)
    );

    assert_eq!(
        Scores::decode(&[1, 0]),
        Err(DecodeError::TooShort {
            expected: 4,
            actual: 2,
        })
    );
}

#[test]
fn rejects_padding_that_is_not_zeros() {
    let mut bytes: Vec<u8> = scores_bytes();
    bytes[15] = 0xFF;
    assert_eq!(Scores::decode(&bytes), Err(DecodeError::InvalidPadding));
}

#[test]
fn messages_hold_maps() {
    let value: WithMaps = WithMaps::default().with_scores(scores());
    let bytes: Vec<u8> = [&[1, 0, 1, 0][..], &scores_bytes()].concat();
    assert_encoding!(WithMaps, value, &bytes);

    let view: WithMapsRef = WithMapsRef::new(&bytes).unwrap();
    assert_eq!(view.scores().unwrap().get("bc"), Some(2));
    assert!(view.rounds().is_none());

    let rounds: WithMaps =
        WithMaps::default().with_rounds(vec![BTreeMap::from([(1, 2)]), BTreeMap::default()]);
    let bytes: Vec<u8> = rounds.encode();
    assert_encoding!(WithMaps, rounds, &bytes);
}

#[test]
fn variants_hold_maps() {
    let value: MapCases = MapCases::Scores(scores());
    let bytes: Vec<u8> = [&[1, 0][..], &scores_bytes()].concat();
    assert_encoding!(MapCases, value.clone(), &bytes);
    assert_eq!(value.as_scores(), Some(&scores()));

    let view: MapCasesRef = MapCasesRef::new(&bytes).unwrap();
    assert_eq!(view.as_scores().unwrap().get("a"), Some(1));
    assert_encoding!(MapCases, MapCases::Scores(BTreeMap::default()), &[1, 0]);
}

#[test]
fn maps_are_objects_in_json() {
    assert_json(&Scores::new(scores()), r#"{"scores":{"a":1,"bc":2}}"#);
    assert_json(
        &WithMaps::default().with_scores(scores()),
        r#"{"scores":{"a":1,"bc":2}}"#,
    );
    assert_json(
        &MapCases::Scores(scores()),
        r#"{"var":"Scores","val":{"a":1,"bc":2}}"#,
    );
}

#[test]
fn simple_keys_are_text_in_json() {
    let mut value: MapKinds = map_kinds();
    value.set_by_point(BTreeMap::default());
    let json: String = serde_json::to_string(&value).unwrap();
    assert!(
        json.contains(r#""by_number":{"1":true,"7":false}"#),
        "{}",
        json
    );
    assert!(json.contains(r#""by_color":{"Red":"red"}"#), "{}", json);
    assert!(
        json.contains(r#""maybe":{"-1":"1.5","5":"NaN"}"#),
        "{}",
        json
    );
    assert_eq!(serde_json::from_str::<MapKinds>(&json).unwrap(), value);
}

#[test]
fn struct_keys_are_not_json() {
    assert!(serde_json::to_string(&map_kinds()).is_err());
}

/// Gets the score of the `name` & the entries of any form of the scores.
fn read_scores<S: ScoresView>(scores: &S, name: &str) -> (Option<u32>, String, usize) {
    let map = scores.scores();
    let entries: Vec<String> = map
        .iter()
        .map(|(name, score)| format!("{}={}", name, score))
        .collect();
    (map.get(&name.to_string()), entries.join(","), map.len())
}

#[test]
fn views_read_maps_of_every_form() {
    let value: Scores = Scores::new(scores());
    let mut bytes: Vec<u8> = scores_bytes();
    let expected = (Some(2), "a=1,bc=2".to_string(), 2);
    assert_eq!(read_scores(&value, "bc"), expected);
    assert_eq!(read_scores(&&value, "bc"), expected);
    assert_eq!(
        read_scores(&ScoresRef::new(&bytes).unwrap(), "bc"),
        expected
    );
    assert_eq!(read_scores(&ScoresBuf::from(&value), "bc"), expected);
    assert_eq!(
        read_scores(&ScoresMut::new(&mut bytes).unwrap(), "bc"),
        expected
    );
    assert_eq!(read_scores(&value, "b").0, None);
    assert_eq!(
        read_scores(&ScoresRef::new(&scores_bytes()).unwrap(), "zz").0,
        None
    );
}

/// Reads a map of each kind from any form.
fn digest<K: MapKindsView>(kinds: &K) -> String {
    let points: Vec<String> = kinds
        .by_point()
        .iter()
        .map(|(point, entry)| format!("{},{}:{}{}", point.x(), point.y(), entry.id(), entry.text()))
        .collect();
    let inner: Option<u8> = kinds.maps().get(&1).and_then(|inner| inner.get(&2));
    format!(
        "{:?} {:?} {} {:?} {:?} {:?}",
        kinds.by_number().get(&7),
        kinds.by_color().get(&Color::Red),
        points.join(";"),
        kinds.lists().get(&"a".to_string()),
        inner,
        kinds.maybe().map(|maybe| maybe.len()),
    )
}

#[test]
fn views_read_maps_of_every_kind() {
    let value: MapKinds = map_kinds();
    let bytes: Vec<u8> = value.encode();
    let expected: String = digest(&value);
    assert_eq!(
        expected,
        "Some(false) Some(\"red\") 1,2:3three;1,3:4 Some([1, 2]) Some(3) Some(2)"
    );
    assert_eq!(digest(&MapKindsRef::new(&bytes).unwrap()), expected);
    assert_eq!(digest(&MapKindsBuf::from(&value)), expected);
}

/// Gets the count of the scores of any form of the cases.
fn scores_len<C: MapCasesView>(cases: &C) -> Option<usize> {
    cases.as_scores().map(|scores| scores.len())
}

#[test]
fn views_read_the_maps_of_variants() {
    let value: MapCases = MapCases::Scores(scores());
    let bytes: Vec<u8> = value.encode();
    assert_eq!(scores_len(&value), Some(2));
    assert_eq!(scores_len(&MapCasesRef::new(&bytes).unwrap()), Some(2));
    assert_eq!(scores_len(&MapCases::Number(1)), None);
}

/// Gets a newer key with each of its fields.
fn newer_key(a: u32, b: u32, c: u32) -> NewerKey {
    NewerKey::default().with_a(a).with_b(b).with_c(c)
}

/// Gets the bytes of a map with the message keys `{ a: 1 }` & `{ a: 256 }`.
fn message_keys_bytes() -> Vec<u8> {
    [
        &[34, 0, 0, 0][..],         // the end of the map
        &[28, 0, 0, 0],             // the end of the keys
        &[2, 0, 0, 0, 16, 0, 0, 0], // the keys: the count & the end of the first
        &[1, 0, 1, 0, 0, 1, 0, 0],  // { a: 256 }
        &[1, 0, 1, 0, 1, 0, 0, 0],  // { a: 1 }
        &[2, 1],                    // the values
    ]
    .concat()
}

#[test]
fn message_keys_are_ordered_by_their_bytes() {
    let one: NewerKey = NewerKey::default().with_a(1);
    let big: NewerKey = NewerKey::default().with_a(256);
    assert!(one < big);
    assert!(one.encode() > big.encode());
    assert_encoding!(
        NewerKeys,
        NewerKeys::new(BTreeMap::from([(one, 1), (big, 2)]), BTreeMap::default()),
        &message_keys_bytes()
    );
}

#[test]
fn rejects_byte_ordered_keys_that_are_not_strictly_increasing() {
    let mut by_value: Vec<u8> = message_keys_bytes();
    by_value[21] = 0;
    by_value[20] = 1;
    by_value[29] = 1;
    by_value[28] = 0;
    by_value[32..].copy_from_slice(&[1, 2]);
    assert_eq!(NewerKeys::decode(&by_value), Err(DecodeError::InvalidMap));

    let mut equal: Vec<u8> = message_keys_bytes();
    equal[28] = 0;
    equal[29] = 1;
    assert_eq!(NewerKeys::decode(&equal), Err(DecodeError::InvalidMap));
}

/// Gets the maps of a newer writer, with keys an older reader orders another way by their values.
fn newer_keys() -> NewerKeys {
    NewerKeys::new(
        BTreeMap::from([
            (newer_key(1, 1, 2), 1),
            (newer_key(1, 2, 1), 2),
            (newer_key(1, 256, 1), 3),
            (NewerKey::default(), 4),
        ]),
        BTreeMap::from([
            (NewerCases::Number(1), 1),
            (NewerCases::Number(256), 2),
            (NewerCases::Text("a".to_string()), 3),
            (NewerCases::Point(Point::new(1, 2)), 4),
            (NewerCases::Point(Point::new(256, 0)), 5),
        ]),
    )
}

#[test]
fn older_readers_take_the_keys_of_newer_writers() {
    let newer: NewerKeys = newer_keys();
    let bytes: Vec<u8> = newer.encode();
    assert_eq!(newer.encoded_len(), bytes.len());
    assert_eq!(NewerKeys::decode(&bytes), Ok(newer));

    let older: OlderKeys = OlderKeys::decode(&bytes).unwrap();
    assert_eq!(older.by_message().len(), 4);
    assert_eq!(older.by_variant().len(), 5);
    assert_eq!(older.encoded_len(), bytes.len());
    assert_eq!(older.encode(), bytes);

    common::assert_one_encoding(&bytes, |changed| {
        let older: OlderKeys = OlderKeys::decode(changed).ok()?;
        assert_eq!(older.encoded_len(), changed.len());
        Some(older.encode())
    });
    common::assert_one_encoding(&bytes, |changed| {
        let newer: NewerKeys = NewerKeys::decode(changed).ok()?;
        assert_eq!(newer.encoded_len(), changed.len());
        Some(newer.encode())
    });
}

#[test]
fn byte_ordered_keys_are_found() {
    let value: NewerKeys = newer_keys();
    let bytes: Vec<u8> = value.encode();
    let keys: NewerKeysRef = NewerKeysRef::new(&bytes).unwrap();
    let map: MapRef<NewerKey, u8> = keys.by_message();
    for (key, number) in value.by_message() {
        let encoded: Vec<u8> = key.encode();
        let key_ref: NewerKeyRef = NewerKeyRef::new(&encoded).unwrap();
        assert_eq!(map.get(key_ref), Some(*number));
        assert_eq!(map.get_by_key(key), Some(*number));
        assert_eq!(MapView::get(&map, key), Some(*number));
        assert_eq!(MapView::get(&value.by_message(), key), Some(*number));
    }
    for (key, number) in value.by_variant() {
        assert_eq!(keys.by_variant().get_by_key(key), Some(*number));
    }
    assert_eq!(map.get_by_key(&newer_key(7, 7, 7)), None);
    assert_eq!(keys.by_variant().get_by_key(&NewerCases::Number(7)), None);
}

#[test]
fn keys_with_a_message_inside_are_ordered_by_their_bytes() {
    let value: NestedKeys = NestedKeys::new(
        BTreeMap::from([
            (
                Tagged::new(1, newer_key(1, 0, 0)),
                Entry::new(1, "abc".to_string()),
            ),
            (
                Tagged::new(256, NewerKey::default()),
                Entry::new(2, "a".to_string()),
            ),
            (
                Tagged::new(2, newer_key(9, 9, 9)),
                Entry::new(3, String::default()),
            ),
        ]),
        BTreeMap::from([
            (vec![newer_key(1, 2, 3)], 1),
            (vec![NewerKey::default(), newer_key(256, 0, 0)], 2),
            (Vec::default(), 3),
        ]),
    );
    let bytes: Vec<u8> = value.encode();
    assert_eq!(value.encoded_len(), bytes.len());
    assert_eq!(NestedKeys::decode(&bytes), Ok(value.clone()));
    common::assert_one_encoding(&bytes, |changed| {
        let nested: NestedKeys = NestedKeys::decode(changed).ok()?;
        assert_eq!(nested.encoded_len(), changed.len());
        Some(nested.encode())
    });

    let nested: NestedKeysRef = NestedKeysRef::new(&bytes).unwrap();
    let ids: Vec<u64> = nested
        .by_struct()
        .keys()
        .iter()
        .map(|key| key.id())
        .collect();
    assert_eq!(ids, [256, 1, 2]);
    let texts: Vec<&str> = nested
        .by_struct()
        .values()
        .iter()
        .map(|entry| entry.text())
        .collect();
    assert_eq!(texts, ["a", "abc", ""]);
    let lists: Vec<&[u8]> = nested
        .by_list()
        .keys()
        .iter()
        .map(|list| list.as_bytes())
        .collect();
    assert!(lists.windows(2).all(|pair| pair[0] < pair[1]));
    for (key, entry) in value.by_struct() {
        let found = nested.by_struct().get_by_key(key).unwrap();
        assert_eq!(found.id(), entry.id());
    }
    for (key, number) in value.by_list() {
        assert_eq!(nested.by_list().get_by_key(key), Some(*number));
    }
}

#[test]
fn encoded_maps_are_equal_by_their_bytes() {
    let bytes: Vec<u8> = scores_bytes();
    let (scores, same): (ScoresRef, ScoresRef) = (
        ScoresRef::new(&bytes).unwrap(),
        ScoresRef::new(&bytes).unwrap(),
    );
    assert_eq!(scores.scores(), same.scores());
    let other: Vec<u8> = Scores::new(BTreeMap::from([("a".to_string(), 2)])).encode();
    assert_ne!(scores.scores(), ScoresRef::new(&other).unwrap().scores());
}
