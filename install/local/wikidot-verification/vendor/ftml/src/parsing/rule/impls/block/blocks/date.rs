/*
 * parsing/rule/impls/block/blocks/date.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use super::prelude::*;
use crate::tree::DateItem;
use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;
use time::format_description::well_known::{Iso8601, Rfc2822, Rfc3339};
use time::{Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};

#[cfg(test)]
use time::macros::{date, datetime};

pub const BLOCK_DATE: BlockRule = BlockRule {
    name: "block-date",
    accepts_names: &["date"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: false,
    parse_fn,
};

fn parse_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing date block (name '{name}', in-head {in_head}, score {flag_score})");
    assert!(!flag_star, "Date doesn't allow star flag");
    assert!(!flag_score, "Date doesn't allow score flag");
    assert_block_name(&BLOCK_DATE, name);

    let (value, mut arguments) = parser.get_head_name_map(&BLOCK_DATE, in_head)?;
    let (format, ago_hover) = split_ago_hover_format(arguments.get("format"));
    let arg_timezone = arguments.get("tz");
    let hover = arguments.get_bool(parser, "hover")?.unwrap_or(false) || ago_hover;

    // Parse out timestamp given by user
    let mut date = parse_date(value)
        .map_err(|_| parser.make_err(ParseErrorKind::BlockMalformedArguments))?;

    if let Some(arg) = arg_timezone {
        // Parse out argument timezone
        let offset = parse_timezone(&arg)
            .map_err(|_| parser.make_err(ParseErrorKind::BlockMalformedArguments))?;

        // Add timezone. If None, then conflicting timezones.
        date = match date.add_timezone(offset) {
            Some(date) => date,
            None => {
                warn!(
                    "Date block has two specified timezones (argument {}, parsed {})",
                    arg.as_ref(),
                    offset,
                );

                return Err(parser.make_err(ParseErrorKind::BlockMalformedArguments));
            }
        };
    }

    // Build and return element
    let element = Element::Date {
        value: date,
        format,
        hover,
    };

    ok!(element)
}

fn split_ago_hover_format<'t>(
    date_format: Option<Cow<'t, str>>,
) -> (Option<Cow<'t, str>>, bool) {
    match date_format {
        Some(Cow::Borrowed(date_format)) => match date_format.strip_suffix("|agohover") {
            Some(display_format) => (Some(Cow::Borrowed(display_format)), true),
            None => (Some(Cow::Borrowed(date_format)), false),
        },
        Some(Cow::Owned(date_format)) => match date_format.strip_suffix("|agohover") {
            Some(display_format) => (Some(Cow::Owned(display_format.to_owned())), true),
            None => (Some(Cow::Owned(date_format)), false),
        },
        None => (None, false),
    }
}

// Parser functions

/// Parse a datetime string and produce its time value, as well as possible timezone info.
fn parse_date(value: &str) -> Result<DateItem, DateParseError> {
    debug!("Parsing possible date value '{value}'");

    // Special case, current time
    if value.eq_ignore_ascii_case("now") || value == "." {
        trace!("Was now");
        return Ok(now().into());
    }

    // Try UNIX timestamp (e.g. 1398763929)
    if let Ok(timestamp) = value.parse::<i64>() {
        trace!("Was UNIX timestamp '{timestamp}'");
        let date =
            OffsetDateTime::from_unix_timestamp(timestamp).map_err(|_| DateParseError)?;

        return Ok(date.into());
    }

    // Try datetime strings
    if let Ok(datetime_tz) = OffsetDateTime::parse(value, &Rfc3339) {
        trace!("Was RFC 3339 datetime string, result '{datetime_tz}'");
        return Ok(datetime_tz.into());
    }

    if let Ok(datetime) = PrimitiveDateTime::parse(value, &Iso8601::PARSING) {
        trace!("Was ISO 8601 datetime string (no timezone), result '{datetime}'");
        return Ok(datetime.into());
    }

    if let Ok(datetime_tz) = OffsetDateTime::parse(value, &Iso8601::PARSING) {
        trace!("Was ISO 8601 datetime string, result '{datetime_tz}'");
        return Ok(datetime_tz.into());
    }

    if let Ok(datetime_tz) = OffsetDateTime::parse(value, &Rfc2822) {
        trace!("Was RFC 2822 datetime string, result '{datetime_tz}'");
        return Ok(datetime_tz.into());
    }

    // Try date strings
    if let Ok(date) = Date::parse(value, &Iso8601::PARSING) {
        trace!("Was ISO 8601 date string, result '{date}'");
        return Ok(date.into());
    }

    // Exhausted all cases, failing
    Err(DateParseError)
}

/// Parse the timezone based on the specifier string.
fn parse_timezone(value: &str) -> Result<UtcOffset, DateParseError> {
    static TIMEZONE_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(\+|-)?([0-9]{1,2}):?([0-9]{2})?$").unwrap());

    debug!("Parsing possible timezone value '{value}'");

    // Try hours / minutes (via regex)
    if let Some(captures) = TIMEZONE_REGEX.captures(value) {
        // Get sign (+1 or -1)
        let sign = match captures.get(1) {
            None => 1,
            Some(mtch) => match mtch.as_str() {
                "+" => 1,
                "-" => -1,
                _ => unreachable!(),
            },
        };

        // Get hour value
        let hour = captures
            .get(2)
            .expect("No hour in timezone despite match")
            .as_str()
            .parse::<i32>()
            .expect("Hour wasn't integer despite match");

        // Get minute value
        let minute = match captures.get(3) {
            None => 0,
            Some(mtch) => mtch
                .as_str()
                .parse::<i32>()
                .expect("Minute wasn't integer despite match"),
        };

        // Get offset in seconds
        let seconds = sign * (hour * 3600 + minute * 60);

        trace!("Was offset via +HH:MM (sign {sign}, hour {hour}, minute {minute})");
        return get_offset(seconds);
    }

    // Try number of seconds
    //
    // This is lower-priority than the regex to permit "integer" cases,
    // such as "0800".
    if let Ok(seconds) = value.parse::<i32>() {
        trace!("Was offset in seconds ({seconds})");
        return get_offset(seconds);
    }

    // Exhausted all cases, failing
    Err(DateParseError)
}

#[derive(Debug, PartialEq, Eq)]
struct DateParseError;

#[inline]
fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

#[inline]
fn get_offset(seconds: i32) -> Result<UtcOffset, DateParseError> {
    UtcOffset::from_whole_seconds(seconds).map_err(|_| DateParseError)
}

// Tests

#[test]
fn date() {
    // Since time will obviously pass between when the time
    // object is created and when we check it, this function
    // makes sure the time is *reasonably close*.
    //
    // This function *will* fail if there's a seam, such as a
    // change to daylight savings or the system clock jumps.
    //
    // Since this is just a test suite, we don't care about such edge
    // cases, just rerun the tests.
    fn dates_equal(date1: DateItem, date2: DateItem) -> bool {
        let timestamp1 = date1.timestamp();
        let timestamp2 = date2.timestamp();

        (timestamp1 - timestamp2).abs() < 5
    }

    macro_rules! test_ok {
        ($input:expr, $date:expr $(,)?) => {{
            let actual = parse_date($input).expect("Datetime parse didn't succeed");
            let expected = $date.into();

            if !dates_equal(actual, expected) {
                panic!(
                    "Actual date value doesn't match expected\nactual: {:?}\nexpected: {:?}",
                    actual,
                    expected,
                );
            }
        }};
    }

    macro_rules! test_err {
        ($input:expr $(,)?) => {{
            parse_date($input).expect_err("Error case for datetime parse succeeded");
        }};
    }

    macro_rules! timestamp {
        ($timestamp:expr $(,)?) => {
            OffsetDateTime::from_unix_timestamp($timestamp)
                .expect("Unable to parse datetime from timestamp")
        };
    }

    test_ok!(".", now());
    test_ok!("now", now());
    test_ok!("Now", now());
    test_ok!("NOW", now());
    test_ok!("1600000000", timestamp!(1600000000));
    test_ok!("-1000", timestamp!(-1000));
    test_ok!("0", timestamp!(0));
    test_ok!("2001-09-11", date!(2001 - 09 - 11));
    test_ok!(
        "2007-05-12T09:34:51.026490",
        datetime!(2007-05-12 09:34:51.026490),
    );
    test_ok!(
        "2007-05-12T09:34:51.026490+04:00",
        datetime!(2007-05-12 09:34:51.026490+04:00),
    );
    test_ok!(
        "2007-05-12T09:34:51.026490-04:00",
        datetime!(2007-05-12 09:34:51.026490-04:00),
    );

    test_err!("");
    test_err!("*");
    test_err!("foobar");
    test_err!("2001-09");
    test_err!("2001/09");
    test_err!("2001/09-11");
    test_err!("2001-09/11");
}

#[test]
fn timezone() {
    macro_rules! test_ok {
        ($input:expr, $offset:expr) => {{
            let actual = parse_timezone(&$input).expect("Timezone parse didn't succeed");

            assert_eq!(
                actual,
                UtcOffset::from_whole_seconds($offset).expect("Invalid timezone offset"),
                "Actual timezone value doesn't match expected",
            );
        }};
    }

    macro_rules! test_err {
        ($input:expr) => {{
            parse_timezone($input).expect_err("Error case for timezone parse succeeded");
        }};
    }

    test_ok!("12345", 12345);
    test_ok!("+12345", 12345);
    test_ok!("-12345", -12345);

    test_ok!("8:00", 8 * 60 * 60);
    test_ok!("+8:00", 8 * 60 * 60);
    test_ok!("-8:00", -8 * 60 * 60);

    test_ok!("08:00", 8 * 60 * 60);
    test_ok!("+08:00", 8 * 60 * 60);
    test_ok!("-08:00", -8 * 60 * 60);

    test_ok!("08:00", 8 * 60 * 60);
    test_ok!("+08:00", 8 * 60 * 60);
    test_ok!("-08:00", -8 * 60 * 60);

    test_ok!("0800", 8 * 60 * 60);
    test_ok!("+0800", 8 * 60 * 60);
    test_ok!("-0800", -8 * 60 * 60);

    test_ok!("800", 8 * 60 * 60);
    test_ok!("+800", 8 * 60 * 60);
    test_ok!("-800", -8 * 60 * 60);

    test_err!("");
    test_err!("*");
    test_err!("8:0");
}

#[test]
fn split_ago_hover_format_removes_suffix_from_display_format() {
    assert_eq!(
        split_ago_hover_format(Some(Cow::Borrowed("%d. %m. %Y|agohover"))),
        (Some(Cow::Owned(String::from("%d. %m. %Y"))), true)
    );
}

#[test]
fn split_ago_hover_format_leaves_normal_format_unchanged() {
    assert_eq!(
        split_ago_hover_format(Some(Cow::Borrowed("%d. %m. %Y"))),
        (Some(Cow::Borrowed("%d. %m. %Y")), false)
    );
}
