/*
 * parsing/collect/consume.rs
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

/// Convenience wrapper around `collect()` to consume each token iteration.
///
/// Since simply consuming to produce an `Element<'t>` is a typical pattern,
/// this function implements it here to avoid code duplication.
///
/// This call always sets `step_on_final` to `true`.
pub fn collect_consume<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
    warn_kind: Option<ParseWarningKind>,
) -> ParseResult<'r, 't, Vec<Element<'t>>> {
    let mut all_elements = Vec::new();

    let (_, exceptions, paragraph_safe) = collect(
        log,
        parser,
        rule,
        close_conditions,
        invalid_conditions,
        warn_kind,
        |log, parser| {
            let success = consume(log, parser)?
                .unwrap_partials(parser)?
                .take(|new_elements| new_elements.append_to(&mut all_elements));

            Ok(success)
        },
    )?
    .into();

    ok!(paragraph_safe; all_elements, exceptions)
}
