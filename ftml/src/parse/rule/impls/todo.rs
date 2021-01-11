/*
 * parse/rule/impls/todo.rs
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

use super::prelude::*;

/// Temporary rule for syntactical constructions that are not yet implemented.
///
/// Will be removed when the first full version of ftml is released.
pub const RULE_TODO: Rule = Rule {
    name: "todo",
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    error!(log, "Encountered unimplemented rule! Returning warning");

    Err(parser.make_warn(ParseWarningKind::NotImplemented))
}
