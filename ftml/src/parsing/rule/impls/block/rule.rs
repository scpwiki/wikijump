/*
 * parsing/rule/impls/block/rule.rs
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
    log: &Logger,
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    trace!(log, "Trying to process a block");

    parse_block(log, parser, false)
}

fn block_special<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    trace!(log, "Trying to process a block (with special)");

    parse_block(log, parser, true)
}

fn block_skip<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
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
        if ![Token::LeftBlock, Token::LeftBlockSpecial].contains(&current.token) {
            return Ok(false);
        }

        // Get the block's name
        let (name, _) = parser.get_block_name(false)?;

        // Get the block rule: if it accepts newlines, then we consume here
        match get_block_rule_with_name(name) {
            Some(block_rule) => Ok(block_rule.accepts_newlines),
            None => Ok(false),
        }
    });

    if result {
        info!(
            log,
            "Skipping newline due to upcoming line-terminated block",
        );

        ok!(Elements::None)
    } else {
        Err(parser.make_warn(ParseWarningKind::RuleFailed))
    }
}

// Block parsing implementation

fn parse_block<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    special: bool,
) -> ParseResult<'r, 't, Elements<'t>>
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

    let (name, in_head) = parser.get_block_name(special)?;
    trace!(log, "Got block name"; "name" => name, "in-head" => in_head);

    let (name, modifier) = match name.strip_suffix('_') {
        Some(name) => (name, true),
        None => (name, false),
    };

    // Get the block rule for this name
    let block = match get_block_rule_with_name(name) {
        Some(block) => block,
        None => return Err(parser.make_warn(ParseWarningKind::NoSuchBlock)),
    };

    // Set block rule for better warnings
    parser.set_block(block);

    // Check if this block allows special invocation (the '[[*' token)
    if !block.accepts_special && special {
        return Err(parser.make_warn(ParseWarningKind::InvalidSpecialBlock));
    }

    // Check if this block allows modifier invocation ('_' after name)
    if !block.accepts_modifier && modifier {
        return Err(parser.make_warn(ParseWarningKind::InvalidModifierBlock));
    }

    parser.get_optional_spaces_any()?;

    // Run the parse function until the end.
    //
    // This is responsible for parsing any arguments,
    // and terminating the block (the ']]' token),
    // then processing the body (if any) and tail block.
    (block.parse_fn)(log, parser, name, special, modifier, in_head)
}
