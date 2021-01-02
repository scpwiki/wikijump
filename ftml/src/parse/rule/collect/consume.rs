/*
 * parse/rule/collect/consume.rs
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

use super::prelude::*;

/// Convenience wrapper around `try_collect()` to consume each token iteration.
///
/// Since simply consuming to produce an `Element<'t>` is a typical pattern,
/// this function implements it here to avoid code duplication.
pub fn try_consume<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
) -> ParseResult<'r, 't, Vec<Element<'t>>> {
    let mut elements = Vec::new();

    let exceptions = try_collect(
        log,
        parser,
        rule,
        close_conditions,
        invalid_conditions,
        |log, parser| {
            consume(log, parser)?.map_ok(|element| {
                elements.push(element);
            })
        },
    )?
    .into();

    ok!(elements, parser.remaining(), exceptions)
}
