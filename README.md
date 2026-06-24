# parse-dms

[![crates.io](https://img.shields.io/crates/v/parse-dms.svg)](https://crates.io/crates/parse-dms)
[![docs.rs](https://docs.rs/parse-dms/badge.svg)](https://docs.rs/parse-dms)
[![CI](https://github.com/trananhtung/parse-dms/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/parse-dms/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/parse-dms.svg)](#license)

**Parse a coordinate string into decimal latitude/longitude.**

`parse-dms` turns a human-written geographic coordinate — in degrees/minutes/seconds or
decimal degrees, with optional hemisphere letters — into decimal degrees:

```rust
use parse_dms::{parse_dms, Dms};

let c = parse_dms("59°12'7.7\"N 002°15'39.6\"W").unwrap();
assert_eq!(c, Dms::Coordinates { lat: Some(59.20213888888889), lon: Some(-2.261) });
```

A faithful Rust port of the [`parse-dms`](https://www.npmjs.com/package/parse-dms) npm
package.

- Accepts varied separators: `°º:d` for degrees, `'’‘′:` for minutes, `"″''` for seconds
- Infers latitude/longitude from hemisphere letters (`N`/`S`/`E`/`W`), or — for two
  hemisphere-less coordinates — from their order (first = lat, second = lon)
- Differential-tested against the reference `parse-dms` implementation

## Install

```toml
[dependencies]
parse-dms = "0.1"
```

## Usage

```rust
use parse_dms::{parse_dms, Dms};

// DMS with hemispheres
assert_eq!(
    parse_dms("48°51'24\"N, 2°21'03\"E").unwrap(),
    Dms::Coordinates { lat: Some(48.85666666666667), lon: Some(2.3508333333333336) }
);

// Decimal pair, order-inferred
assert_eq!(
    parse_dms("40.7128, -74.0060").unwrap(),
    Dms::Coordinates { lat: Some(40.7128), lon: Some(-74.006) }
);

// Single coordinate with a hemisphere → only that axis is set
assert_eq!(parse_dms("51 N").unwrap(), Dms::Coordinates { lat: Some(51.0), lon: None });

// A bare value with no hemisphere → a single decimal number
assert_eq!(parse_dms("-51.5").unwrap(), Dms::Decimal(-51.5));

// Out-of-range and unparseable inputs error
assert!(parse_dms("181").is_err());
assert!(parse_dms("nonsense").is_err());
```

## Return value

`parse_dms` returns a [`Dms`]:

- `Dms::Decimal(f64)` — a single coordinate that had no hemisphere letter.
- `Dms::Coordinates { lat, lon }` — otherwise; each field is `None` when the input didn't
  provide that component.

Components are validated: degrees `0..=180`, minutes and seconds `0..=60`.

## Compatibility note

This is a faithful port, including the reference's quirks — for example a leading
hemisphere letter triggers a one-character overlap when scanning for a second coordinate
(`parse_dms("n51")` yields `{ lat: 51, lon: 1 }`, matching the npm package). Lowercase
hemisphere letters are, as in the reference, not treated as hemispheres.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
