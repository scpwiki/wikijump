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
    let hover = arguments.get_bool(parser, "hover")?;

    // Parse out timestamp given by user
    let (naive_datetime, parsed_timezone) = parse_date(log, value)
        .map_err(|_| parser.make_warn(ParseWarningKind::BlockMalformedArguments))?;

    // Check that two timezones weren't passed, only one can be used
    if arg_timezone.is_some() && parsed_timezone.is_some() {
        warn!(
            log,
            "Date block has two specified timezones";
            "argument-timezone" => arg_timezone.unwrap(),
            "parsed-timezone" => str!(parsed_timezone.unwrap()),
        );

        return Err(parser.make_warn(ParseWarningKind::BlockMalformedArguments));
    }

    todo!()
}

fn parse_date(
    log: &Logger,
    value: &str,
) -> Result<(NaiveDateTime, Option<FixedOffset>), DateParseError> {
    info!(log, "Parsing possible date value"; "value" => value);

    // First, check if it's a UNIX timestamp (e.g. 1398763929)
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
    if let Ok(date) = NaiveDateTime::parse_from_str(value, "%F %T") {
        debug!(
            log,
            "Was ISO 8601 datetime string";
            "result" => str!(date),
        );

        return Ok((date, None));
    }

    // Try full RFC 2822 datetime string
    if let Ok(tz_date) = DateTime::parse_from_rfc2822(value) {
        debug!(
            log,
            "Was RFC 2822 datetime string";
            "result" => str!(tz_date),
        );

        let date = tz_date.naive_utc();
        let timezone = tz_date.timezone();

        return Ok((date, Some(timezone)));
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

#[derive(Debug)]
struct DateParseError;
