/*
 * parsing/rule/impls/block/blocks/bibliography.rs
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
use crate::tree::{Bibliography, DefinitionListItem};

pub const BLOCK_BIBLIOGRAPHY: BlockRule = BlockRule {
    name: "block-bibliography",
    accepts_names: &["bibliography"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        "Parsing bibliography block (name '{name}', in-head {in_head}, score {flag_score})"
    );
    assert!(!flag_star, "Bibliography doesn't allow star flag");
    assert!(!flag_score, "Bibliography doesn't allow score flag");
    assert_block_name(&BLOCK_BIBLIOGRAPHY, name);

    let mut arguments = parser.get_head_map(&BLOCK_BIBLIOGRAPHY, in_head)?;

    let title = arguments.get("title");
    let hide = arguments.get_bool(parser, "hide")?.unwrap_or(false);

    // Get body content. The contents should only be a definition list, but
    // we use the regular elements parser to make it easy on us. If we find
    // anything else, we fail the rule.
    //
    // We also discard paragraph_safe, since it's not relevant, and this element
    // never is (uses <div>).
    let (elements, errors, _) =
        parser.get_body_elements(&BLOCK_BIBLIOGRAPHY, false)?.into();

    // Build up the bibliography
    //
    // Look through to find definition lists, ignoring "space" type elements,
    // and adding definition list values to the bibliography as we find them.
    let mut bibliography = Bibliography::new();

    for element in elements {
        match element {
            // Append definition list entries
            Element::DefinitionList(items) => {
                for DefinitionListItem {
                    key_string,
                    value_elements,
                    ..
                } in items
                {
                    bibliography.add(key_string, value_elements);
                }
            }

            // Skip whitespace elements
            _ if element.is_whitespace() => continue,

            // Other elements
            _ => {
                warn!(
                    "Received non-definition list within bibliography block: {}",
                    element.name(),
                );

                return Err(parser
                    .make_err(ParseErrorKind::BibliographyContainsNonDefinitionList));
            }
        }
    }

    // Add bibliography object to parser for unified tracking, like footnotes.
    let index = parser.push_bibliography(bibliography);

    ok!(Element::BibliographyBlock { index, title, hide }, errors)
}
