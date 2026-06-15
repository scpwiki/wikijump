/*
 * parsing/rule/impls/block/blocks/iftags.rs
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
use crate::data::PageInfo;
use crate::parsing::ElementCondition;

pub const BLOCK_IFTAGS: BlockRule = BlockRule {
    name: "block-iftags",
    accepts_names: &["iftags"],
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
    debug!("Parsing iftags block (name '{name}', in-head {in_head})");
    assert!(!flag_star, "IfTags doesn't allow star flag");
    assert!(!flag_score, "IfTags doesn't allow score flag");
    assert_block_name(&BLOCK_IFTAGS, name);

    // Parse out tag conditions
    let conditions =
        parser.get_head_value(&BLOCK_IFTAGS, in_head, |parser, spec| match spec {
            Some(spec) => Ok(ElementCondition::parse(spec)),
            None => Err(parser.make_err(ParseErrorKind::BlockMissingArguments)),
        })?;

    let include_body = check_iftags(parser.page_info(), &conditions);

    // Get body content, never with paragraphs. If the condition fails, skip the
    // body as text so inactive branches cannot produce parse errors.
    let (elements, errors, paragraph_safe) = if include_body {
        parser.get_body_elements(&BLOCK_IFTAGS, false)?.into()
    } else {
        parser.get_body_text(&BLOCK_IFTAGS)?;
        (Vec::new(), Vec::new(), true)
    };

    trace!(
        "IfTags conditions parsed (conditions length {}, elements length {})",
        conditions.len(),
        elements.len(),
    );

    // Return elements based on condition
    let elements = if include_body {
        trace!("Conditions passed, including elements");

        Elements::Multiple(elements)
    } else {
        trace!("Conditions failed, excluding elements");

        Elements::None
    };

    ok!(paragraph_safe; elements, errors)
}

pub fn check_iftags(info: &PageInfo, conditions: &[ElementCondition]) -> bool {
    trace!("Checking iftags");
    ElementCondition::check(conditions, &info.tags)
}
