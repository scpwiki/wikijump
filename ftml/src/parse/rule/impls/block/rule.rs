/*
 * parse/rule/impls/block/rule.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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

use super::super::prelude::*;
use super::mapping::get_block_rule_with_name;
use super::BlockParser;

pub const RULE_BLOCK: Rule = Rule {
    name: "block",
    try_consume_fn: block_regular,
};

pub const RULE_BLOCK_SPECIAL: Rule = Rule {
    name: "block-special",
    try_consume_fn: block_special,
};

// Rule implementations

fn block_regular<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(log, "Trying to process a block");

    parse_block(log, parser, false)
}

fn block_special<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(log, "Trying to process a block (with special)");

    parse_block(log, parser, true)
}

// Block parsing implementation

fn parse_block<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
    special: bool,
) -> ParseResult<'r, 't, Element<'t>>
where
    'r: 't,
{
    debug!(
        log,
        "Trying to process a block";
        "special" => special,
    );

    let mut parser = BlockParser::new(log, parser, special);

    // Get block name
    parser.get_optional_space()?;

    let (name, in_block) = parser.get_block_name()?;

    // Get the block rule for this name
    let block = match get_block_rule_with_name(name) {
        Some(block) => block,
        None => return Err(parser.make_error(ParseErrorKind::NoSuchBlock)),
    };

    // Check if this block allows special invocation (the '[[*' token)
    if !block.accepts_special && special {
        return Err(parser.make_error(ParseErrorKind::InvalidSpecialBlock));
    }

    // Prepare to run the block's parsing function
    parser.set_block(block);
    parser.get_optional_space()?;

    // Run the parse function until the end.
    // This is responsible for parsing any arguments,
    // and terminating the block (the ']]' token),
    // then processing the body (if any) and close tag.
    (block.parse_fn)(log, &mut parser, name, special, in_block)
}
