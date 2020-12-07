/*
 * parse/rule/impls/link.rs
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
use crate::enums::AnchorTarget;

pub const RULE_LINK: Rule = Rule {
    name: "link",
    try_consume_fn: link,
};

pub const RULE_LINK_TAB: Rule = Rule {
    name: "link",
    try_consume_fn: link_new_tab,
};

fn link<'t, 'r>(
    log: &slog::Logger,
    _: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'t, 'r> {
    trace!(log, "Trying to create a bare link (regular)");

    let (extracted, remaining) = remaining
        .split_first()
        .expect("On link token but there are no futher extractions");

    try_consume_link(log, extracted, remaining, full_text, AnchorTarget::Same)
}

fn link_new_tab<'t, 'r>(
    log: &slog::Logger,
    _: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'t, 'r> {
    trace!(log, "Trying to create a bare link (new tab)");

    let (extracted, remaining) = remaining
        .split_first()
        .expect("On link token but there are no futher extractions");

    try_consume_link(log, extracted, remaining, full_text, AnchorTarget::NewTab)
}

fn try_consume_link<'t, 'r>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
    target: AnchorTarget,
) -> Consumption<'t, 'r> {
    debug!(log, "Trying to create a bare link"; "target" => target.name());

    todo!()
}
