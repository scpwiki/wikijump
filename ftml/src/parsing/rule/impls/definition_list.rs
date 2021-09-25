/*
 * parsing/rule/impls/definition_list.rs
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
use std::borrow::Cow;

type DefinitionItem<'t> = (Cow<'t, str>, Cow<'t, str>);

pub const RULE_DEFINITION_LIST: Rule = Rule {
    name: "definition-list",
    position: LineRequirement::StartOfLine,
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    info!(log, "Trying to create a definition list");

    let mut items = Vec::new();

    // Definition list must have at least one pair
    match parse_item(log, parser)? {
        Some(item) => items.push(item),
        None => return Err(parser.make_warn(ParseWarningKind::RuleFailed)),
    }

    // Add the rest of the pairs
    while let Some(item) = parse_item(log, parser)? {
        items.push(item);
    }

    // Build and return element
    ok!(Element::DefinitionList { items })
}

fn parse_item<'t>(
    log: &Logger,
    parser: &mut Parser<'_, 't>,
) -> Result<Option<DefinitionItem<'t>>, ParseWarning> {
    debug!(log, "Trying to parse a definition list item pair");

    todo!()
}
