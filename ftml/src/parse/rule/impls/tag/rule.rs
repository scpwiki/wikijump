/*
 * parse/rule/impls/tag/rule.rs
 *
 * ftml - Library to parse Wikidot text
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

use super::super::prelude::*;

pub const RULE_TAG: Rule = Rule {
    name: "tag",
    try_consume_fn: tag_regular,
};

pub const RULE_TAG_SPECIAL: Rule = Rule {
    name: "tag-special",
    try_consume_fn: tag_special,
};

fn tag_regular<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    trace!(log, "Trying to process a tag");

    tag(log, extracted, remaining, full_text, false)
}

fn tag_special<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    trace!(log, "Trying to process a tag (with special)");

    tag(log, extracted, remaining, full_text, true)
}

fn tag<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
    special: bool,
) -> Consumption<'r, 't> {
    debug!(
        log,
        "Trying to process a tag (special: {})",
        special;
        "special" => special,
    );

    todo!()
}
