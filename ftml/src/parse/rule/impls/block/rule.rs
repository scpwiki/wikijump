/*
 * parse/rule/impls/block/rule.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2020 Ammon Smith
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
use super::mapping::block_with_name;
use super::BlockParser;
use crate::parse::UpcomingTokens;
use crate::tree::Element;

pub const RULE_BLOCK: Rule = Rule {
    name: "block",
    try_consume_fn: block_regular,
};

pub const RULE_BLOCK_SPECIAL: Rule = Rule {
    name: "block-special",
    try_consume_fn: block_special,
};

// Rule implementations

fn block_regular<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    trace!(log, "Trying to process a block");

    parse_block(log, extracted, remaining, full_text, false)
}

fn block_special<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    trace!(log, "Trying to process a block (with special)");

    parse_block(log, extracted, remaining, full_text, true)
}

// Block parsing implementation

fn parse_block<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
    special: bool,
) -> Consumption<'r, 't>
where
    'r: 't,
{
    match parse_block_internal(log, extracted, remaining, full_text, special) {
        Ok(outcome) => outcome.into(),
        Err(error) => Consumption::err(error),
    }
}

fn parse_block_internal<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
    special: bool,
) -> Result<BlockParseOutcome<'r, 't>, ParseError>
where
    'r: 't,
{
    debug!(
        log,
        "Trying to process a block (special: {})",
        special;
        "special" => special,
    );

    let mut parser = BlockParser::new(log, special, extracted, remaining, full_text);

    // Get block name
    parser.get_optional_space()?;

    let name = parser.get_identifier(ParseErrorKind::BlockMissingName)?;

    // Get the block rule for this name
    let block = match block_with_name(name) {
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
    // then terminating the block (the ']]' token),
    // then processing the body (if any) and close tag.

    let (element, exceptions) = match (block.parse_fn)(log, &mut parser, name, special) {
        Consumption::Failure { error } => return Err(error),
        Consumption::Success {
            item,
            remaining,
            exceptions,
        } => {
            parser.update(remaining)?;

            (item, exceptions)
        }
    };

    // Finished parsing, return outcome
    let remaining = parser.into_remaining();

    Ok(BlockParseOutcome {
        element,
        remaining,
        exceptions,
    })
}

#[derive(Debug)]
struct BlockParseOutcome<'r, 't> {
    element: Element<'t>,
    remaining: &'r [ExtractedToken<'t>],
    exceptions: Vec<ParseException<'t>>,
}

impl<'r, 't> Into<Consumption<'r, 't>> for BlockParseOutcome<'r, 't> {
    #[inline]
    fn into(self) -> Consumption<'r, 't> {
        let BlockParseOutcome {
            element,
            remaining,
            exceptions,
        } = self;

        Consumption::warn(element, remaining, exceptions)
    }
}
