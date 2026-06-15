/*
 * parsing/collect/text.rs
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

/// Generic function to consume all tokens into a single string slice.
///
/// This is a subset of the functionality provided by `collect()`,
/// as it specifically gathers all the extracted tokens into a string slice,
/// rather than considering them as special elements.
#[inline]
pub fn collect_text<'p, 'r, 't>(
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
    error_kind: Option<ParseErrorKind>,
) -> Result<&'t str, ParseError>
where
    'r: 't,
{
    collect_text_keep(
        parser,
        rule,
        close_conditions,
        invalid_conditions,
        error_kind,
    )
    .map(|(slice, _)| slice)
}

/// Modified form of `collect_text()` that also returns the last token.
///
/// The last token terminating the collection is kept, and returned
/// to the caller alongside the string slice.
///
/// Compare with `collect_consume_keep()`.
pub fn collect_text_keep<'p, 'r, 't>(
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    close_conditions: &[ParseCondition],
    invalid_conditions: &[ParseCondition],
    error_kind: Option<ParseErrorKind>,
) -> Result<(&'t str, &'r ExtractedToken<'t>), ParseError>
where
    'r: 't,
{
    // Log collect_text() call
    debug!("Trying to consume tokens to merge into a single string");

    let (start, mut end) = (parser.current(), None);

    // Iterate and collect the tokens to merge.
    //
    // We know text is always paragraph safe, so we ignore that value.
    let (last, errors, _) = collect(
        parser,
        rule,
        close_conditions,
        invalid_conditions,
        error_kind,
        |parser| {
            trace!("Ingesting token in string span");

            end = Some(parser.current());
            ok!(true; ())
        },
    )?
    .into();

    assert!(
        errors.is_empty(),
        "Exceptions were returned during text token collection",
    );

    let slice = match (start, end) {
        // We have a token span, use to get string slice
        (start, Some(end)) => parser.full_text().slice(start, end),

        // Empty list of tokens, resultant slice must be empty
        (_, None) => "",
    };

    Ok((slice, last))
}
