/*
 * parsing/rule/impls/block/blocks/date.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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
use crate::tree::Date;
use chrono::prelude::*;
use regex::Regex;

pub const BLOCK_DATE: BlockRule = BlockRule {
    name: "block-date",
    accepts_names: &["date"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: false,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    info!(
        log,
        "Parsing date block";
        "flag-score" => flag_score,
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "Date doesn't allow star flag");
    assert!(!flag_score, "Date doesn't allow score flag");
    assert_block_name(&BLOCK_DATE, name);

    let (value, mut arguments) = parser.get_head_name_map(&BLOCK_DATE, in_head)?;
    let format = arguments.get("format");
    let arg_timezone = arguments.get("tz");
    let hover = arguments.get_bool(parser, "hover")?.unwrap_or(true);

    // Parse out timestamp given by user
    let mut date = parse_date(log, value)
        .map_err(|_| parser.make_warn(ParseWarningKind::BlockMalformedArguments))?;

    if let Some(arg) = arg_timezone {
        // Parse out argument timezone
        let offset = parse_timezone(log, &arg)
            .map_err(|_| parser.make_warn(ParseWarningKind::BlockMalformedArguments))?;

        // Add timezone. If None, then conflicting timezones.
        date = match date.add_timezone(offset) {
            Some(date) => date,
            None => {
                warn!(
                    log,
                    "Date block has two specified timezones";
                    "argument-timezone" => arg.as_ref(),
                    "parsed-timezone" => str!(offset),
                );

                return Err(parser.make_warn(ParseWarningKind::BlockMalformedArguments));
            }
        };
    }

    // Build and return element
    let element = Element::Date {
        time: date,
        format,
        hover,
    };

    ok!(element)
}

// Parser functions

/// Parse a datetime string and produce its time value, as well as possible timezone info.
fn parse_date(log: &Logger, value: &str) -> Result<Date, DateParseError> {
    info!(log, "Parsing possible date value"; "value" => value);

    // Special case, current time
    if value.eq_ignore_ascii_case("now") || value == "." {
        debug!(log, "Was now");

        return Ok(now());
    }

    // Try UNIX timestamp (e.g. 1398763929)
    if let Ok(timestamp) = value.parse::<i64>() {
        debug!(log, "Was UNIX timestamp"; "timestamp" => timestamp);

        let date = NaiveDateTime::from_timestamp(timestamp, 0);

        return Ok(date.into());
    }

    // Try date strings
    if let Ok(date) = NaiveDate::parse_from_str(value, "%F") {
        debug!(
            log,
            "Was ISO 8601 date string (dashes)";
            "result" => str!(date),
        );

        return Ok(date.into());
    }

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y/%m/%d") {
        debug!(
            log,
            "Was ISO 8601 date string (slashes)";
            "result" => str!(date),
        );

        return Ok(date.into());
    }

    // Try datetime strings
    if let Ok(datetime) = NaiveDateTime::parse_from_str(value, "%FT%T") {
        debug!(
            log,
            "Was ISO 8601 datetime string (dashes)";
            "result" => str!(datetime),
        );

        return Ok(datetime.into());
    }

    if let Ok(datetime) = NaiveDateTime::parse_from_str(value, "%Y/%m/%dT%T") {
        debug!(
            log,
            "Was ISO 8601 datetime string (slashes)";
            "result" => str!(datetime),
        );

        return Ok(datetime.into());
    }

    // Try full RFC 3339 (stricter form of ISO 8601)
    if let Ok(datetime_tz) = DateTime::parse_from_rfc3339(value) {
        debug!(
            log,
            "Was RFC 3339 datetime string";
            "result" => str!(datetime_tz),
        );

        return Ok(datetime_tz.into());
    }

    // Exhausted all cases, failing
    Err(DateParseError)
}

/// Parse the timezone based on the specifier string.
fn parse_timezone(log: &Logger, value: &str) -> Result<FixedOffset, DateParseError> {
    lazy_static! {
        static ref TIMEZONE_REGEX: Regex =
            Regex::new(r"^(\+|-)?([0-9]{1,2}):?([0-9]{2})?$").unwrap();
    }

    info!(log, "Parsing possible timezone value"; "value" => value);

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

        debug!(
            log,
            "Was offset via +HH:MM";
            "sign" => sign,
            "hour" => hour,
            "minute" => minute,
            "offset" => seconds,
        );

        return Ok(FixedOffset::east(seconds));
    }

    // Try number of seconds
    //
    // This is lower-priority than the regex to permit "integer" cases,
    // such as "0800".
    if let Ok(seconds) = value.parse::<i32>() {
        debug!(
            log,
            "Was offset in seconds";
            "seconds" => seconds,
        );

        return Ok(FixedOffset::east(seconds));
    }

    // Exhausted all cases, failing
    Err(DateParseError)
}

#[derive(Debug, PartialEq, Eq)]
struct DateParseError;

#[inline]
fn now() -> Date {
    Utc::now().naive_utc().into()
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
    fn dates_equal(date1: Date, date2: Date) -> bool {
        let timestamp1 = date1.timestamp();
        let timestamp2 = date2.timestamp();

        (timestamp1 - timestamp2).abs() < 5
    }

    let log = crate::build_logger();

    macro_rules! check_ok {
        ($input:expr, $date:expr $(,)?) => {{
            let actual = parse_date(&log, $input).expect("Datetime parse didn't succeed");
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

    macro_rules! check_err {
        ($input:expr $(,)?) => {{
            parse_date(&log, $input)
                .expect_err("Error case for datetime parse succeeded");
        }};
    }

    check_ok!(".", now());
    check_ok!("now", now());
    check_ok!("Now", now());
    check_ok!("NOW", now());
    check_ok!("1600000000", NaiveDateTime::from_timestamp(1600000000, 0),);
    check_ok!("-1000", NaiveDateTime::from_timestamp(-1000, 0));
    check_ok!("0", NaiveDateTime::from_timestamp(0, 0));
    check_ok!("2001-09-11", NaiveDate::from_ymd(2001, 09, 11),);
    check_ok!(
        "2001-09-11T08:46:00",
        NaiveDate::from_ymd(2001, 09, 11).and_hms(8, 46, 0),
    );
    check_ok!("2001/09/11", NaiveDate::from_ymd(2001, 09, 11),);
    check_ok!(
        "2001/09/11T08:46:00",
        NaiveDate::from_ymd(2001, 09, 11).and_hms(8, 46, 0),
    );
    check_ok!(
        "2007-05-12T09:34:51.026490+04:00",
        DateTime::from_utc(
            NaiveDate::from_ymd(2007, 05, 12).and_hms_micro(5, 34, 51, 26490),
            FixedOffset::east(4 * 60 * 60),
        ),
    );
    check_ok!(
        "2007-05-12T09:34:51.026490-04:00",
        DateTime::from_utc(
            NaiveDate::from_ymd(2007, 05, 12).and_hms_micro(13, 34, 51, 26490),
            FixedOffset::west(4 * 60 * 60),
        ),
    );

    check_err!("");
    check_err!("*");
    check_err!("foobar");
    check_err!("2001-09");
    check_err!("2001/09");
    check_err!("2001/09-11");
    check_err!("2001-09/11");
}

#[test]
fn timezone() {
    let log = crate::build_logger();

    macro_rules! check_ok {
        ($input:expr, $offset:expr) => {{
            let actual =
                parse_timezone(&log, $input).expect("Timezone parse didn't succeed");

            assert_eq!(
                actual,
                FixedOffset::east($offset),
                "Actual timezone value doesn't match expected",
            );
        }};
    }

    macro_rules! check_err {
        ($input:expr) => {{
            parse_timezone(&log, $input)
                .expect_err("Error case for timezone parse succeeded");
        }};
    }

    check_ok!("12345", 12345);
    check_ok!("+12345", 12345);
    check_ok!("-12345", -12345);

    check_ok!("8:00", 8 * 60 * 60);
    check_ok!("+8:00", 8 * 60 * 60);
    check_ok!("-8:00", -8 * 60 * 60);

    check_ok!("08:00", 8 * 60 * 60);
    check_ok!("+08:00", 8 * 60 * 60);
    check_ok!("-08:00", -8 * 60 * 60);

    check_ok!("08:00", 8 * 60 * 60);
    check_ok!("+08:00", 8 * 60 * 60);
    check_ok!("-08:00", -8 * 60 * 60);

    check_ok!("0800", 8 * 60 * 60);
    check_ok!("+0800", 8 * 60 * 60);
    check_ok!("-0800", -8 * 60 * 60);

    check_ok!("800", 8 * 60 * 60);
    check_ok!("+800", 8 * 60 * 60);
    check_ok!("-800", -8 * 60 * 60);

    check_err!("");
    check_err!("*");
    check_err!("8:0");
}
