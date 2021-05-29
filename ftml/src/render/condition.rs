/*
 * render/condition.rs
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

use crate::log::prelude::*;
use crate::tree::ElementCondition;
use crate::PageInfo;

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

#[inline]
pub fn check_iftags(
    log: &Logger,
    info: &PageInfo,
    conditions: &[ElementCondition],
) -> bool {
    debug!(
        log,
        "Checking iftags";
        "tags-len" => info.tags.len(),
        "conditions-len" => conditions.len(),
    );

    ElementCondition::check(conditions, &info.tags)
}
