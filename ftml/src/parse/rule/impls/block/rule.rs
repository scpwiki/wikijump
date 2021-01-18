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

pub const RULE_BLOCK: Rule = Rule {
    name: "block",
    try_consume_fn: block_regular,
};

pub const RULE_BLOCK_SPECIAL: Rule = Rule {
    name: "block-special",
    try_consume_fn: block_special,
};

pub const RULE_BLOCK_SKIP: Rule = Rule {
    name: "block-skip",
    try_consume_fn: block_skip,
};

// Rule implementations

fn block_regular<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(log, "Trying to process a block");

    parse_block(log, parser, false)
}

fn block_special<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(log, "Trying to process a block (with special)");

    parse_block(log, parser, true)
}

fn block_skip<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(
        log,
        "Trying to see if we skip a newline due to upcoming block",
    );

    assert_eq!(
        parser.current().token,
        Token::LineBreak,
        "Trying to skip because block, but current is not line break",
    );

    let current = parser.step()?;

    // See if there's a block upcoming
    let result = parser.evaluate_fn(|parser| {
        // Make sure this is the start of a block
        if current.token != Token::LeftBlock && current.token != Token::LeftBlockSpecial {
            return Ok(false);
        }

        // Get the block's name
        let (name, _) = parser.get_block_name()?;

        // Get the associated block rule
        let block = match get_block_rule_with_name(name) {
            Some(block) => block,
            None => return Ok(false),
        };

        // Now, if it wants newlines, ignore this newline.
        // The rule will succeed.
        //
        // If it doesn't, let the rule fail. Then it will pass on to a fallback.
        Ok(block.newline_separator)
    });

    if result {
        ok!(Element::Null)
    } else {
        Err(parser.make_warn(ParseWarningKind::RuleFailed))
    }
}

// Block parsing implementation

fn parse_block<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
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

    // Set general rule based on presence of special
    parser.set_rule(if special {
        RULE_BLOCK_SPECIAL
    } else {
        RULE_BLOCK
    });

    // Get block name
    parser.get_optional_space()?;

    let (name, in_head) = parser.get_block_name()?;
    trace!(log, "Got block name"; "name" => name, "in-head" => in_head);

    // Get the block rule for this name
    let block = match get_block_rule_with_name(name) {
        Some(block) => block,
        None => return Err(parser.make_warn(ParseWarningKind::NoSuchBlock)),
    };

    // Check if this block allows special invocation (the '[[*' token)
    if !block.accepts_special && special {
        return Err(parser.make_warn(ParseWarningKind::InvalidSpecialBlock));
    }

    // Prepare to run the block's parsing function
    parser.set_block(block);
    parser.get_optional_space()?;

    // Run the parse function until the end.
    //
    // This is responsible for parsing any arguments,
    // and terminating the block (the ']]' token),
    // then processing the body (if any) and tail block.
    (block.parse_fn)(log, parser, name, special, in_head)
}
