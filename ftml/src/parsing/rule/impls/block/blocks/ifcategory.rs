/*
 * parsing/rule/impls/block/blocks/ifcategory.rs
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
use crate::data::PageInfo;
use crate::parsing::{ElementCondition, ElementConditionType};

pub const BLOCK_IFCATEGORY: BlockRule = BlockRule {
    name: "block-ifcategory",
    accepts_names: &["ifcategory"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing ifcategory block";
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "IfCategory doesn't allow star flag");
    assert!(!flag_score, "IfCategory doesn't allow score flag");
    assert_block_name(&BLOCK_IFCATEGORY, name);

    // Parse out tag conditions
    let conditions =
        parser.get_head_value(&BLOCK_IFCATEGORY, in_head, |parser, spec| match spec {
            None => Err(parser.make_warn(ParseWarningKind::BlockMissingArguments)),
            Some(spec) => {
                let mut conditions = ElementCondition::parse(spec);

                conditions.iter_mut().for_each(|condition| {
                    // Because a page can be in at most one category,
                    // the required condition type is not useful here
                    // beyond a single instance.
                    //
                    // Thus, we convert all required -> present,
                    // effectively making "+" and no prefix the same thing.
                    if condition.ctype == ElementConditionType::Required {
                        condition.ctype = ElementConditionType::Present;
                    }
                });

                Ok(conditions)
            }
        })?;

    // Get body content, never with paragraphs
    let (elements, mut exceptions, paragraph_safe) =
        parser.get_body_elements(&BLOCK_IFCATEGORY, false)?.into();

    debug!(
        log,
        "IfCategory conditions parsed";
        "conditions" => format!("{:#?}", conditions),
        "elements-len" => elements.len(),
    );

    // Return elements based on condition
    let elements = if check_ifcategory(log, parser.page_info(), &conditions) {
        trace!(log, "Conditions passed, including elements");

        Elements::Multiple(elements)
    } else {
        trace!(log, "Conditions failed, excluding elements");

        // Filter out non-warning exceptions
        exceptions.retain(|ex| matches!(ex, ParseException::Warning(_)));

        Elements::None
    };

    ok!(paragraph_safe; elements, exceptions)
}

pub fn check_ifcategory(
    log: &Logger,
    info: &PageInfo,
    conditions: &[ElementCondition],
) -> bool {
    let category = match &info.category {
        Some(category) => category,
        None => "_default",
    };

    debug!(
        log,
        "Checking ifcategory";
        "category" => category,
        "conditions-len" => conditions.len(),
    );

    ElementCondition::check(conditions, &[cow!(category)])
}
