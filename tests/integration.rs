//! Integration tests exercising the public API of `parse-dms`.

use parse_dms::{parse_dms, Dms, ParseError};

#[test]
fn dms_pairs() {
    assert_eq!(
        parse_dms("48°51'24\"N, 2°21'03\"E").unwrap(),
        Dms::Coordinates { lat: Some(48.85666666666667), lon: Some(2.3508333333333336) }
    );
    assert_eq!(
        parse_dms("51°28'40.12\"N, 0°00'05.31\"W").unwrap(),
        Dms::Coordinates { lat: Some(51.477811111111116), lon: Some(-0.001475) }
    );
}

#[test]
fn decimal_and_single() {
    assert_eq!(
        parse_dms("40.7128, -74.0060").unwrap(),
        Dms::Coordinates { lat: Some(40.7128), lon: Some(-74.006) }
    );
    assert_eq!(parse_dms("90 S 180 W").unwrap(), Dms::Coordinates { lat: Some(-90.0), lon: Some(-180.0) });
    assert_eq!(parse_dms("180").unwrap(), Dms::Decimal(180.0));
}

#[test]
fn errors() {
    assert_eq!(parse_dms("181"), Err(ParseError::DegreesOutOfRange));
    assert_eq!(parse_dms("1°61'"), Err(ParseError::MinutesOutOfRange));
    assert_eq!(parse_dms("not a coordinate at all !!!"), Err(ParseError::CouldNotParse));
}
