/*
 * parsing/rule/impls/fallback.rs
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

pub const RULE_FALLBACK: Rule = Rule {
    name: "fallback",
    position: LineRequirement::Any,
    try_consume_fn,
};

/// The stubbed implementation of `try_consume` for `RULE_FALLBACK`.
///
/// This is a special case, since we never "try" to consume a fallback,
/// it always activates when all other rules fail. As such, it is never
/// executed directly.
///
/// See the end of the `consume()` function in `parse/consume.rs` for
/// where the fallback action is performed.
fn try_consume_fn<'r, 't>(_: &mut Parser<'r, 't>) -> ParseResult<'r, 't, Elements<'t>> {
    panic!("Manual fallback rule should not be executed directly!")
}
