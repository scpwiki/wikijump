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

pub const RULE_BLOCK: Rule = Rule {
    name: "block",
    try_consume_fn: block_regular,
};

pub const RULE_BLOCK_SPECIAL: Rule = Rule {
    name: "block-special",
    try_consume_fn: block_special,
};

fn block_regular<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    trace!(log, "Trying to process a block");

    into_consumption(block(log, extracted, remaining, full_text, false))
}

fn block_special<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    trace!(log, "Trying to process a block (with special)");

    into_consumption(block(log, extracted, remaining, full_text, true))
}

fn block<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
    special: bool,
) -> Result<(), ParseError> {
    debug!(
        log,
        "Trying to process a block (special: {})",
        special;
        "special" => special,
    );

    let mut parser = BlockParser::new(log, special, extracted, remaining, full_text);

    // Get block name
    parser.get_optional_space()?;

    let name = parser.get_identifier()?;

    // Get the block rule for this name
    let block = match block_with_name(name) {
        Some(block) => block,
        None => return Err(parser.make_error(ParseErrorKind::NoSuchBlock)),
    };

    parser.set_block(block);

    todo!()
}

fn into_consumption<'r, 't>(result: Result<(), ParseError>) -> Consumption<'r, 't> {
    match result {
        Ok(_idk) => todo!(),
        Err(error) => Consumption::err(error),
    }
}
