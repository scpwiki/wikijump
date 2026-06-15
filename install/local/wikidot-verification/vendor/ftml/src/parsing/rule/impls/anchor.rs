/*
 * parsing/rule/impls/anchor.rs
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

//! Rule for anchor name blocks.
//!
//! Not to be confused with the anchor block (`[[a]]`), this
//! "block" is a rule for `[[# name-of-anchor]]`, that is, created an
//! `<a id="name-of-anchor">` anchor that can be jumped to.

use super::prelude::*;
use crate::id_prefix::isolate_ids;
use std::borrow::Cow;

pub const RULE_ANCHOR: Rule = Rule {
    name: "anchor",
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Trying to create a named anchor");
    assert_step(parser, Token::LeftBlockAnchor)?;

    // Requires a space before the name
    parser.get_token(Token::Whitespace, ParseErrorKind::RuleFailed)?;

    // Gather name for anchor
    let name = collect_text(
        parser,
        RULE_ANCHOR,
        &[ParseCondition::current(Token::RightBlock)],
        &[
            ParseCondition::current(Token::Whitespace),
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    // Isolate ID if requested
    let name = if parser.settings().isolate_user_ids {
        Cow::Owned(isolate_ids(name))
    } else {
        cow!(name)
    };

    // Build and return link element
    ok!(Element::AnchorName(name))
}
