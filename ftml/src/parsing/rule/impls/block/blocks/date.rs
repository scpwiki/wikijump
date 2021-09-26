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
use chrono::prelude::*;
use regex::Regex;

pub const BLOCK_DATE: BlockRule = BlockRule {
    name: "block-date",
    accepts_names: &["date"],
    accepts_star: false,
    accepts_score: true,
    accepts_newlines: true,
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
    let (naive_datetime, parsed_timezone) = parse_date(log, value)
        .map_err(|_| parser.make_warn(ParseWarningKind::BlockMalformedArguments))?;

    // Get timezone info
    let timezone = match (arg_timezone, parsed_timezone) {
        // Check that two timezones weren't passed, only one can be used
        (Some(arg), Some(parsed)) => {
            warn!(
                log,
                "Date block has two specified timezones";
                "argument-timezone" => arg.as_ref(),
                "parsed-timezone" => str!(parsed),
            );

            return Err(parser.make_warn(ParseWarningKind::BlockMalformedArguments));
        }

        // Argument-specified timezone
        (Some(tz), None) => parse_timezone(log, &tz)
            .map_err(|_| parser.make_warn(ParseWarningKind::BlockMalformedArguments))?,

        // String-specified timezone
        (None, Some(tz)) => tz,

        // No specified timezone, use UTC
        (None, None) => utc(),
    };

    // Build timezone-aware datetime
    let datetime = DateTime::<FixedOffset>::from_utc(naive_datetime, timezone);

    // Build and return element
    let element = Element::Date {
        time: datetime,
        format,
        hover,
    };

    ok!(element)
}

/// Parse a datetime string and produce its time value, as well as possible timezone info.
fn parse_date(
    log: &Logger,
    value: &str,
) -> Result<(NaiveDateTime, Option<FixedOffset>), DateParseError> {
    info!(log, "Parsing possible date value"; "value" => value);

    // Special case, current time
    if value.eq_ignore_ascii_case("now") || value == "." {
        debug!(log, "Was now");

        // This looks weird, but it's just "current time, but no timezone info"
        let date = Utc::now().naive_utc();

        return Ok((date, None));
    }

    // Try UNIX timestamp (e.g. 1398763929)
    if let Ok(timestamp) = value.parse::<i64>() {
        debug!(log, "Was UNIX timestamp"; "timestamp" => timestamp);

        let date = NaiveDateTime::from_timestamp(timestamp, 0);

        return Ok((date, None));
    }

    // Try date string
    if let Ok(date) = NaiveDateTime::parse_from_str(value, "%F") {
        debug!(
            log,
            "Was ISO 8601 date string";
            "result" => str!(date),
        );

        return Ok((date, None));
    }

    // Try datetime string
    if let Ok(date) = NaiveDateTime::parse_from_str(value, "%FT%T") {
        debug!(
            log,
            "Was ISO 8601 datetime string";
            "result" => str!(date),
        );

        return Ok((date, None));
    }

    // Try full RFC 3339 (stricter form of ISO 8601)
    if let Ok(tz_date) = DateTime::parse_from_rfc3339(value) {
        debug!(
            log,
            "Was RFC 3339 datetime string";
            "result" => str!(tz_date),
        );

        let date = tz_date.naive_utc();
        let timezone = tz_date.timezone();

        return Ok((date, Some(timezone)));
    }

    // Exhausted all cases, failing
    Err(DateParseError)
}

/// Parse the timezone based on the specifier string.
fn parse_timezone(log: &Logger, value: &str) -> Result<FixedOffset, DateParseError> {
    lazy_static! {
        static ref TIMEZONE_REGEX: Regex =
            Regex::new(r"(\+|-)?([0-9]{2}):?([0-9]{2})?").unwrap();
    }

    info!(log, "Parsing possible timezone value"; "value" => value);

    // Try number of seconds
    if let Ok(seconds) = value.parse::<i32>() {
        debug!(
            log,
            "Was offset in seconds";
            "seconds" => seconds,
        );

        return Ok(FixedOffset::east(seconds));
    }

    // Try hours / minutes (via regex)
    if let Some(captures) = TIMEZONE_REGEX.captures(value) {
        // Get sign (+1 or -1)
        let sign = match captures.get(0) {
            None => 1,
            Some(mtch) => match mtch.as_str() {
                "+" => 1,
                "-" => -1,
                _ => unreachable!(),
            },
        };

        // Get hour value
        let hour = captures
            .get(1)
            .expect("No hour in timezone despite match")
            .as_str()
            .parse::<i32>()
            .expect("Hour wasn't integer despite match");

        // Get minute value
        let minute = match captures.get(2) {
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

    // Exhausted all cases, failing
    Err(DateParseError)
}

#[derive(Debug)]
struct DateParseError;

/// Helper function to get a `FixedOffset`-equivalent of `Utc`.
///
/// This exists to make the code more clear, since this actual
/// construction looks weird.
#[inline]
fn utc() -> FixedOffset {
    FixedOffset::east(0)
}
