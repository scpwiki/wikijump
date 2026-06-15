/*
 * parsing/rule/impls/block/blocks/bibcite.rs
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

pub const BLOCK_BIBCITE: BlockRule = BlockRule {
    name: "block-bibcite",
    accepts_names: &["bibcite"],
    accepts_star: false,
    accepts_score: true,
    accepts_newlines: false,
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
        "Parsing bibcite block (name '{name}', in-head {in_head}, score {flag_score})",
    );
    assert!(!flag_star, "Bibcite doesn't allow star flag");
    assert_block_name(&BLOCK_BIBCITE, name);

    let label =
        parser.get_head_value(&BLOCK_BIBCITE, in_head, |parser, value| match value {
            Some(value) => Ok(value.trim()),
            None => {
                warn!("No label provided in [[bibcite]], failing rule");
                Err(parser.make_err(ParseErrorKind::BlockMissingArguments))
            }
        })?;

    // "bibcite" means we wrap it in brackets
    // "bibcite_" means it's bare, like ((bibcite))
    let brackets = !flag_score;

    ok!(Element::BibliographyCite {
        label: cow!(label),
        brackets,
    })
}
