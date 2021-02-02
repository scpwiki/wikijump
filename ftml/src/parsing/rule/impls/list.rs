/*
 * parsing/rule/impls/list.rs
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
use crate::enums::ListStyle;

pub const RULE_BULLET_LIST: Rule = Rule {
    name: "bullet-list",
    try_consume_fn: bullet,
};

pub const RULE_NUMBERED_LIST: Rule = Rule {
    name: "numbered-list",
    try_consume_fn: number,
};

fn bullet<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(log, "Consuming tokens to build a bullet list");

    parse_list(log, parser, RULE_BULLET_LIST, Token::BulletItem, ListStyle::Bullet)
}

fn number<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(log, "Consuming tokens to build a numbered list");

    parse_list(log, parser, RULE_NUMBERED_LIST, Token::NumberedItem, ListStyle::Numbered)
}

fn parse_list<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    bullet_token: Token,
    list_style: ListStyle,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(
        log,
        "Parsing a list";
        "rule" => rule.name(),
        "bullet-token" => bullet_token,
        "list-style" => list_style.name(),
    );

    assert!(
        parser.current().token == Token::InputStart ||
        parser.current().token == Token::LineBreak,
        "Starting token for list is not start of input or newline",
    );
    parser.step()?;

    // Produce a depth list with elements
    let mut depths = Vec::new();
    let mut exceptions = Vec::new();

    loop {
        let depth = match parser.current().token {
            // Count the number of spaces for its depth
            Token::Whitespace => {
                let spaces = parser.current().slice;
                parser.step()?;

                // Since these are only ASCII spaces a byte count is fine
                spaces.len()
            },

            // No depth, just the bullet
            token if token == bullet_token => 0,

            // Invalid token, bail
            _ => break,
        };

        // Check that we're processing the right bullet
        if parser.current().token != bullet_token {
            break;
        }

        // For now, always expect whitespace after the bullet
        if parser.step()?.token != Token::Whitespace {
            break;
        }

        // Parse elements until we hit the end of the line
        let elements = collect_consume(
            log,
            parser,
            rule,
            &[ParseCondition::current(Token::LineBreak), ParseCondition::current(Token::InputEnd)],
            &[ParseCondition::current(Token::ParagraphBreak)],
            None,
        )?
        .chain(&mut exceptions);

        // Append bullet line
        depths.push((depth, elements));
    }
println!("depths: {:#?}", depths);

    // Our rule is in another castle
    if depths.is_empty() {
        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
    }

    todo!()
}
