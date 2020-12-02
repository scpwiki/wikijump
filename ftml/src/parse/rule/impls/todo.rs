/*
 * parse/rule/impls/todo.rs
 *
 * ftml - Library to parse Wikidot code
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

/// Temporary rule for syntactical constructions that are not yet implemented.
pub const RULE_TODO: Rule = Rule {
    name: "todo",
    try_consume_fn,
};

fn try_consume_fn<'t, 'r>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    _: &'r [ExtractedToken<'t>],
) -> Consumption<'t, 'r> {
    error!(log, "Encountered unimplemented rule! Returning error");

    Consumption::err(ParseError::new(
        ParseErrorKind::NotImplemented,
        RULE_TODO,
        extracted,
    ))
}
