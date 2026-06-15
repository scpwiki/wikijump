/*
 * parsing/rule/impls/variable.rs
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
use regex::Regex;
use std::sync::LazyLock;

static VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\$(.+)\}").unwrap());

pub const RULE_VARIABLE: Rule = Rule {
    name: "variable",
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Consuming token by placing variable contents");

    let ExtractedToken { slice, .. } = parser.current();

    let variable = VARIABLE_REGEX
        .captures(slice)
        .expect("Variable regex didn't match")
        .get(1)
        .expect("Capture group not found")
        .as_str();

    ok!(Element::Variable(cow!(variable)))
}
