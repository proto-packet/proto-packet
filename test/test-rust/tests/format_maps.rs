mod common;

use common::assert_json;
use proto_packet::io::{DecodeError, MapRef};
use proto_packet::types::Float64;
use proto_packet_test::format::examples::{Color, Entry, Point};
use proto_packet_test::format::maps::{
    MapCases, MapCasesRef, MapKinds, MapKindsRef, Scores, ScoresRef, WithMaps, WithMapsRef,
};
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
            ("a".to_string(), vec![1, 2]),
            ("b".to_string(), Vec::default()),
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
    assert_eq!(kinds.lists().get("a").unwrap().as_bytes(), [1, 2]);
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
