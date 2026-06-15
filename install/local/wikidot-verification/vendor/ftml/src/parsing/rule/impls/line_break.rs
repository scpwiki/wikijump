/*
 * parsing/rule/impls/line_break.rs
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

use super::prelude::*;

pub const RULE_LINE_BREAK: Rule = Rule {
    name: "line-break",
    position: LineRequirement::Any,
    try_consume_fn: line_break,
};

pub const RULE_LINE_BREAK_PARAGRAPH: Rule = Rule {
    name: "line-break-paragraph",
    position: LineRequirement::Any,
    try_consume_fn: line_break_paragraph,
};

fn line_break<'r, 't>(parser: &mut Parser<'r, 't>) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Consuming newline token as line break");

    // Skip this newline if we're coming up on a rule that starts
    // on its own line.
    //
    // Grep for "LineRequirement::StartOfLine" and compare that with this list.

    let upcoming_skip = parser.evaluate_fn(|parser| {
        parser.step()?;
        parser.get_optional_space()?;

        Ok(matches!(
            parser.current().token,
            Token::Quote
                | Token::BulletItem
                | Token::NumberedItem
                | Token::Heading
                | Token::Equals
                | Token::TripleDash
                | Token::ClearFloatLeft
                | Token::ClearFloatRight
                | Token::ClearFloatBoth
                | Token::TableColumn
                | Token::TableColumnRight
                | Token::TableColumnCenter
                | Token::TableColumnTitle
        ))
    });

    if upcoming_skip {
        debug!("Skipping line break element because of upcoming token");
        return ok!(Elements::None);
    }

    ok!(Element::LineBreak)
}

#[inline]
fn line_break_paragraph<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    // This rule is kind of special. It's the same as RULE_LINE_BREAK,
    // except it accepts a *ParagraphBreak* instead, which is normally supposed to split
    // paragraphs.
    //
    // So what's going on?
    //
    // In "normal" contexts, you extract paragraphs (see gather_paragraphs()), so any
    // Token::ParagraphBreak tokens are used to affected the ParagraphStack and create
    // a new paragraph container.
    //
    // However other contexts do not allow new paragraphs to form, such as [[span]].
    // In these cases, if we encounter two or more newlines, we must pretend it's simply
    // one regular newline, or a line break.

    line_break(parser)
}
