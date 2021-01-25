/*
 * parsing/rule/impls/block/blocks/lines.rs
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
use std::num::NonZeroU32;

pub const BLOCK_LINES: BlockRule = BlockRule {
    name: "block-lines",
    accepts_names: &["lines", "newlines"],
    accepts_special: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    special: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(log, "Parsing newlines block"; "in-head" => in_head);

    assert_eq!(special, false, "Code doesn't allow special variant");
    assert_block_name(&BLOCK_LINES, name);

    let count = parser.get_head_value(&BLOCK_LINES, in_head, parse_count)?;

    ok!(Element::LineBreaks(count))
}

fn parse_count<'r, 't>(
    parser: &Parser<'r, 't>,
    argument: Option<&'t str>,
) -> Result<NonZeroU32, ParseWarning> {
    let argument = match argument {
        Some(arg) => arg.trim(),
        None => return Err(parser.make_warn(ParseWarningKind::BlockMissingArguments)),
    };

    argument.parse().map_err(|error| {
        debug!(&parser.log(), "Invalid numeric expression: {}", error);

        parser.make_warn(ParseWarningKind::BlockMalformedArguments)
    })
}
