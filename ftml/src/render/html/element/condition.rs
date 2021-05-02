/*
 * render/html/element/condition.rs
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
use crate::tree::ElementCondition;

pub fn render_iftags(
    log: &Logger,
    ctx: &mut HtmlContext,
    conditions: &[ElementCondition],
    elements: &[Element],
) {
    debug!(
        log,
        "Rendering iftags element";
        "conditions-len" => conditions.len(),
    );

    let tags = &ctx.info().tags;
    if conditions.iter().all(|cond| cond.check(tags)) {
        debug!(
            log,
            "All conditions passed, rendering";
            "elements-len" => elements.len(),
        );

        render_elements(log, ctx, elements);
    }
}
