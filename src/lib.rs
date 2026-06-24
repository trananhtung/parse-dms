//! # parse-dms — parse coordinate strings into decimal latitude/longitude
//!
//! Parse a geographic coordinate written in degrees/minutes/seconds (or decimal degrees),
//! with optional hemisphere letters, into decimal degrees:
//!
//! ```
//! use parse_dms::{parse_dms, Dms};
//!
//! let c = parse_dms("59°12'7.7\"N 002°15'39.6\"W").unwrap();
//! assert_eq!(c, Dms::Coordinates { lat: Some(59.20213888888889), lon: Some(-2.261) });
//!
//! // A bare value with no hemisphere returns a single decimal number.
//! assert_eq!(parse_dms("-51.5").unwrap(), Dms::Decimal(-51.5));
//! ```
//!
//! A faithful Rust port of the [`parse-dms`](https://www.npmjs.com/package/parse-dms) npm
//! package. Accepts a wide range of punctuation for the degree/minute/second separators
//! (`°º:d`, `'’‘′`, `"″''`), inferring latitude/longitude from hemisphere letters or, when
//! there are two hemisphere-less coordinates, from their order.

#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/parse-dms/0.1.0")]
// The decimal-degree literals below are exact reference values; separators would obscure them.
#![allow(clippy::unreadable_literal)]

use regex_lite::{Captures, Regex};
use std::fmt;
use std::sync::OnceLock;

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// The result of parsing a coordinate string.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dms {
    /// A single coordinate with no hemisphere — just a decimal-degree value.
    Decimal(f64),
    /// A latitude and/or longitude in decimal degrees. A field is `None` when the input did
    /// not supply that component.
    Coordinates {
        /// Decimal latitude, if present.
        lat: Option<f64>,
        /// Decimal longitude, if present.
        lon: Option<f64>,
    },
}

/// An error from [`parse_dms`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The input did not contain a recognizable coordinate.
    CouldNotParse,
    /// Degrees were outside the range `0..=180`.
    DegreesOutOfRange,
    /// Minutes were outside the range `0..=60`.
    MinutesOutOfRange,
    /// Seconds were outside the range `0..=60`.
    SecondsOutOfRange,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ParseError::CouldNotParse => "could not parse string",
            ParseError::DegreesOutOfRange => "degrees out of range",
            ParseError::MinutesOutOfRange => "minutes out of range",
            ParseError::SecondsOutOfRange => "seconds out of range",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Lat,
    Lon,
}

impl Axis {
    fn opposite(self) -> Axis {
        match self {
            Axis::Lat => Axis::Lon,
            Axis::Lon => Axis::Lat,
        }
    }
}

fn dms_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?i)([NSEW])?\s?(-)?(\d+(?:\.\d+)?)[°º:d\s]?\s?(?:(\d+(?:\.\d+)?)['’‘′:]?\s?(?:(\d{1,2}(?:\.\d+)?)(?:"|″|’’|'')?)?)?\s?([NSEW])?"#,
        )
        .expect("static regex is valid")
    })
}

/// The hemisphere/sign letters, matched case-sensitively against the capture (mirroring the
/// reference's uppercase lookup table).
fn hemisphere_sign(letter: &str) -> Option<f64> {
    match letter {
        "N" | "E" => Some(1.0),
        "S" | "W" => Some(-1.0),
        _ => None,
    }
}

fn hemisphere_axis(letter: &str) -> Option<Axis> {
    match letter {
        "N" | "S" => Some(Axis::Lat),
        "E" | "W" => Some(Axis::Lon),
        _ => None,
    }
}

/// Compute `{ decDeg, axis }` from a regex match.
///
/// `suppress_trailing` mirrors the reference clearing `m1[6] = undefined` when the match
/// began with a hemisphere letter.
fn dec_deg_from_match(
    m: &Captures<'_>,
    suppress_trailing: bool,
) -> Result<(f64, Option<Axis>), ParseError> {
    let g = |i: usize| m.get(i).map(|x| x.as_str());
    let g6 = if suppress_trailing { None } else { g(6) };

    // sign = signIndex[m[2]] || signIndex[m[1]] || signIndex[m[6]] || 1
    let sign = if g(2) == Some("-") {
        -1.0
    } else {
        g(1).and_then(hemisphere_sign)
            .or_else(|| g6.and_then(hemisphere_sign))
            .unwrap_or(1.0)
    };

    let degrees: f64 = g(3).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let minutes: f64 = g(4).map_or(0.0, |s| s.parse().unwrap_or(0.0));
    let seconds: f64 = g(5).map_or(0.0, |s| s.parse().unwrap_or(0.0));

    let axis = g(1)
        .and_then(hemisphere_axis)
        .or_else(|| g6.and_then(hemisphere_axis));

    if !(0.0..=180.0).contains(&degrees) {
        return Err(ParseError::DegreesOutOfRange);
    }
    if !(0.0..=60.0).contains(&minutes) {
        return Err(ParseError::MinutesOutOfRange);
    }
    if !(0.0..=60.0).contains(&seconds) {
        return Err(ParseError::SecondsOutOfRange);
    }

    let dec = sign * (degrees + minutes / 60.0 + seconds / 3600.0);
    Ok((dec, axis))
}

/// Substring of `s` starting at UTF-16 code-unit index `start` (mirroring JS `substr`).
fn utf16_substr(s: &str, start: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().collect();
    if start >= units.len() {
        return String::new();
    }
    String::from_utf16_lossy(&units[start..])
}

/// Parse a coordinate string into decimal degrees.
///
/// Returns [`Dms::Decimal`] for a single hemisphere-less coordinate, otherwise
/// [`Dms::Coordinates`] with the inferred latitude and/or longitude.
///
/// # Errors
/// Returns [`ParseError`] if the string cannot be parsed or a component is out of range.
///
/// ```
/// # use parse_dms::{parse_dms, Dms};
/// assert_eq!(parse_dms("51 N").unwrap(), Dms::Coordinates { lat: Some(51.0), lon: None });
/// assert!(parse_dms("foo").is_err());
/// ```
pub fn parse_dms(input: &str) -> Result<Dms, ParseError> {
    let s = input.trim();
    let re = dms_regex();

    let m1 = re.captures(s).ok_or(ParseError::CouldNotParse)?;
    let leading_hemisphere = m1.get(1).is_some();
    let (dec1, axis1_match) = dec_deg_from_match(&m1, leading_hemisphere)?;

    // Find where the second coordinate begins, replicating `substr(m1[0].length [- 1])`.
    let matched = m1.get(0).map_or("", |x| x.as_str());
    let utf16_len = matched.encode_utf16().count();
    let start = if leading_hemisphere {
        utf16_len.saturating_sub(1)
    } else {
        utf16_len
    };
    let rest = utf16_substr(s, start);
    let s2 = rest.trim();

    let dec2 = match re.captures(s2) {
        Some(c2) => Some(dec_deg_from_match(&c2, false)?),
        None => None,
    };

    let axis2_match = dec2.and_then(|(_, a)| a);
    let (axis1, axis2) = if let Some(a1) = axis1_match {
        (a1, axis2_match.unwrap_or_else(|| a1.opposite()))
    } else if dec2.is_none() {
        return Ok(Dms::Decimal(dec1));
    } else {
        // Two hemisphere-less coordinates → first is lat, second is lon (by order).
        (Axis::Lat, Axis::Lon)
    };

    let mut lat = None;
    let mut lon = None;
    assign(&mut lat, &mut lon, axis1, dec1);
    if let Some((d2, _)) = dec2 {
        assign(&mut lat, &mut lon, axis2, d2);
    }
    Ok(Dms::Coordinates { lat, lon })
}

fn assign(lat: &mut Option<f64>, lon: &mut Option<f64>, axis: Axis, value: f64) {
    match axis {
        Axis::Lat => *lat = Some(value),
        Axis::Lon => *lon = Some(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dms_pair() {
        assert_eq!(
            parse_dms("59°12'7.7\"N 002°15'39.6\"W").unwrap(),
            Dms::Coordinates {
                lat: Some(59.20213888888889),
                lon: Some(-2.261)
            }
        );
    }

    #[test]
    fn decimal_pair_inferred_order() {
        assert_eq!(
            parse_dms("59.20213 -2.260987").unwrap(),
            Dms::Coordinates {
                lat: Some(59.20213),
                lon: Some(-2.260987)
            }
        );
    }

    #[test]
    fn single_with_hemisphere() {
        assert_eq!(
            parse_dms("51 N").unwrap(),
            Dms::Coordinates {
                lat: Some(51.0),
                lon: None
            }
        );
        assert_eq!(
            parse_dms("51 E").unwrap(),
            Dms::Coordinates {
                lat: None,
                lon: Some(51.0)
            }
        );
    }

    #[test]
    fn single_decimal() {
        assert_eq!(parse_dms("51").unwrap(), Dms::Decimal(51.0));
        assert_eq!(parse_dms("-51.5").unwrap(), Dms::Decimal(-51.5));
        // lowercase hemisphere is not recognized → a bare number
        assert_eq!(parse_dms("51n").unwrap(), Dms::Decimal(51.0));
    }

    #[test]
    fn colon_and_d_separators() {
        assert_eq!(
            parse_dms("51:30:00 N 000:30:00 E").unwrap(),
            Dms::Coordinates {
                lat: Some(51.5),
                lon: Some(0.5)
            }
        );
        assert_eq!(
            parse_dms("51d30N").unwrap(),
            Dms::Coordinates {
                lat: Some(51.5),
                lon: None
            }
        );
    }

    #[test]
    fn out_of_range_and_unparseable() {
        assert_eq!(parse_dms("181"), Err(ParseError::DegreesOutOfRange));
        assert_eq!(parse_dms("12°61'N"), Err(ParseError::MinutesOutOfRange));
        assert_eq!(parse_dms("foo"), Err(ParseError::CouldNotParse));
        assert_eq!(parse_dms(""), Err(ParseError::CouldNotParse));
    }

    #[test]
    fn leading_hemisphere_overlap() {
        // The reference's `substr(len - 1)` re-reads a digit: "n51" → {lat:51, lon:1}.
        assert_eq!(
            parse_dms("n51").unwrap(),
            Dms::Coordinates {
                lat: Some(51.0),
                lon: Some(1.0)
            }
        );
    }
}
