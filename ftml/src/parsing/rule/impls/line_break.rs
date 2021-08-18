/*
 * parsing/rule/impls/line_break.rs
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

fn line_break<'p, 'r, 't>(
    log: &Logger,
    _parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Consuming newline token as line break");

    ok!(Element::LineBreak)
}

fn line_break_paragraph<'p, 'r, 't>(
    log: &Logger,
    _parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Consuming paragraph break as line break");

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

    ok!(Element::LineBreak)
}
