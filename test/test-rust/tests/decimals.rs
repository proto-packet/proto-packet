mod common;

use common::assert_json;
use proto_packet::types::Decimal;
use proto_packet_test::format::edges::{Prices, PricesMut, PricesRef};
use proto_packet_test::format::messages::AllKinds;
use proto_packet_test::format::variants::AllCases;
use std::collections::{BTreeMap, HashSet};

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

/// Gets a struct with decimals.
fn prices() -> Prices {
    Prices::new(
        Decimal::new(150),
        Decimal::new(1),
        Decimal::new(7),
        None,
        Vec::default(),
        BTreeMap::default(),
    )
}

#[test]
fn decimals_are_their_units() {
    let bytes: Vec<u8> = [
        &[150, 0, 0, 0, 0, 0, 0, 0][..], // price, 1.50
        &[1, 0, 0, 0, 0, 0, 0, 0],       // rate, 0.000001
        &[7, 0, 0, 0, 0, 0, 0, 0],       // count, 7
        &[0; 8],                         // maybe, absent
        &[37, 0, 0, 0],                  // the end of the list
        &[0],                            // bits
    ]
    .concat();
    assert_eq!(bytes.len(), 37);
    assert_encoding!(Prices, prices(), &bytes);

    let view: PricesRef = PricesRef::new(&bytes).unwrap();
    assert_eq!(view.price().units(), 150);
    assert_eq!(view.price().to_string(), "1.50");
    assert_eq!(view.rate().to_string(), "0.000001");
    assert_eq!(view.count().to_string(), "7");
    assert_eq!(view.maybe(), None);
}

#[test]
fn decimals_round_trip_everywhere() {
    let value: Prices = Prices::new(
        Decimal::new(i64::MIN),
        Decimal::new(i64::MAX),
        Decimal::new(-7),
        Some(Decimal::new(-5)),
        vec![Decimal::new(1), Decimal::new(-1)],
        BTreeMap::from([("a".to_string(), Decimal::new(199))]),
    );
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(Prices, value, &bytes);

    let bytes: [u8; 12] = [1, 0, 14, 0, 150, 0, 0, 0, 0, 0, 0, 0];
    assert_encoding!(
        AllKinds,
        AllKinds::default().with_price(Decimal::new(150)),
        &bytes
    );
    let bytes: [u8; 10] = [15, 0, 0xFB, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    assert_encoding!(AllCases, AllCases::Price(Decimal::new(-5)), &bytes);
}

#[test]
fn mut_sets_decimals_in_place() {
    let mut bytes: Vec<u8> = prices().encode();
    let mut view: PricesMut = PricesMut::new(&mut bytes).unwrap();
    view.set_price(Decimal::new(-250));
    view.set_maybe(Decimal::new(1));
    let decoded: Prices = Prices::decode(&bytes).unwrap();
    assert_eq!(decoded.price().to_string(), "-2.50");
    assert_eq!(decoded.maybe(), Some(Decimal::new(1)));
}

#[test]
fn decimals_write_all_their_fraction_digits() {
    let text = |units: i64| Decimal::<2>::new(units).to_string();
    assert_eq!(text(150), "1.50");
    assert_eq!(text(5), "0.05");
    assert_eq!(text(-5), "-0.05");
    assert_eq!(text(0), "0.00");
    assert_eq!(text(i64::MAX), "92233720368547758.07");
    assert_eq!(text(i64::MIN), "-92233720368547758.08");
    assert_eq!(Decimal::<0>::new(-42).to_string(), "-42");
    assert_eq!(Decimal::<18>::new(1).to_string(), "0.000000000000000001");
    assert_eq!(format!("{:?}", Decimal::<2>::new(150)), "1.50");
}

#[test]
fn decimals_parse_their_text() {
    let parse = |text: &str| text.parse::<Decimal<2>>().map(Decimal::units);
    assert_eq!(parse("1.50"), Ok(150));
    assert_eq!(parse("1.5"), Ok(150));
    assert_eq!(parse("1"), Ok(100));
    assert_eq!(parse("-0.05"), Ok(-5));
    assert_eq!(parse("-0"), Ok(0));
    assert_eq!(parse("007.10"), Ok(710));
    assert_eq!(parse("92233720368547758.07"), Ok(i64::MAX));
    assert_eq!(parse("-92233720368547758.08"), Ok(i64::MIN));
    assert_eq!("42".parse::<Decimal<0>>().map(Decimal::units), Ok(42));
    for text in [
        "",
        "-",
        "1.",
        ".5",
        "+1",
        "1e2",
        " 1",
        "1 ",
        "1.505",
        "1.5.0",
        "1,5",
        "0x1",
        "--1",
        "NaN",
        "92233720368547758.08",
        "-92233720368547758.09",
        "99999999999999999999999999999999999999999",
    ] {
        assert!(parse(text).is_err(), "{:?}", text);
    }
    assert!("1.5".parse::<Decimal<0>>().is_err());
}

#[test]
fn decimals_have_an_identity() {
    let a: Decimal<2> = Decimal::new(150);
    let b: Decimal<2> = "1.5".parse().unwrap();
    assert_eq!(a, b);
    assert!(Decimal::<2>::new(-1) < Decimal::new(0));
    assert_eq!(HashSet::from([a, b]).len(), 1);
    assert_eq!(Decimal::<2>::default().units(), 0);
    assert_eq!(Decimal::<2>::SCALE, 2);
}

#[test]
fn decimals_add_and_subtract() {
    let a: Decimal<2> = Decimal::new(150);
    assert_eq!(a.checked_add(Decimal::new(25)), Some(Decimal::new(175)));
    assert_eq!(a.checked_sub(Decimal::new(200)), Some(Decimal::new(-50)));
    assert_eq!(
        Decimal::<2>::new(i64::MAX).checked_add(Decimal::new(1)),
        None
    );
    assert_eq!(
        Decimal::<2>::new(i64::MIN).checked_sub(Decimal::new(1)),
        None
    );
}

#[test]
fn decimals_are_text_in_json() {
    let mut value: Prices = prices();
    value.set_maybe(Decimal::new(-5));
    value.set_list(vec![Decimal::new(100)]);
    value.set_named(BTreeMap::from([("a".to_string(), Decimal::new(199))]));
    assert_json(
        &value,
        r#"{"price":"1.50","rate":"0.000001","count":"7","maybe":"-0.05","list":["1.00"],"named":{"a":"1.99"}}"#,
    );
    assert_json(
        &AllCases::Price(Decimal::new(150)),
        r#"{"var":"Price","val":"1.50"}"#,
    );
    assert_eq!(
        serde_json::from_str::<AllCases>(r#"{"val":"1.5","var":"Price"}"#).unwrap(),
        AllCases::Price(Decimal::new(150))
    );
}

#[test]
fn json_rejects_invalid_decimals() {
    for json in [
        r#"{"var":"Price","val":1.5}"#,     // a plain number
        r#"{"var":"Price","val":1}"#,       // a plain whole number
        r#"{"var":"Price","val":"1.505"}"#, // too many fraction digits
        r#"{"var":"Price","val":"1e2"}"#,   // an exponent
        r#"{"var":"Price","val":""}"#,      // empty text
    ] {
        assert!(serde_json::from_str::<AllCases>(json).is_err(), "{}", json);
    }
}
