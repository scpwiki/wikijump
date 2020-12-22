/*
 * parse/rule/collect/paragraph.rs
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
use crate::parse::gather_paragraphs;
use crate::parse::upcoming::UpcomingTokens;
use crate::tree::{Container, ContainerType, Element};

pub fn try_paragraph<'t, 'r>(
    log: &slog::Logger,
    (extracted, remaining, full_text): (
        &'r ExtractedToken<'t>,
        &'r [ExtractedToken<'t>],
        FullText<'t>,
    ),
    rule: Rule,
    close_tokens: &[Token],
    invalid_tokens: &[Token],
) -> GenericConsumption<'t, 'r, Vec<Element<'t>>> {
    // Log try_paragraph() call
    info!(
        log,
        "Trying to consume tokens to produce paragraph for {:?}", rule,
    );

    // Iterate and consume the tokens into multiple elements
    let mut tokens = UpcomingTokens::from((extracted, remaining));
    //gather_paragraphs(log, tokens, full_text, rule, close_tokens, invalid_tokens)

    // Collapse the ParseStack into a paragraph
    match stack.build_paragraph() {
        Some(paragraph) => {
            debug!(
                log,
                "Finished building paragraph, returning successful consumption",
            );

            Consumption::ok(paragraph, tokens.slice())
        }
        None => {
            debug!(
                log,
                "Attempt at building paragraph yielded no child elements",
            );

            if allow_empty {
                // Build empty paragraph element to return in consumption success
                let container = Container::new(ContainerType::Paragraph, Vec::new());
                let paragraph = Element::Container(container);

                Consumption::ok(paragraph, tokens.slice())
            } else {
                Consumption::err(ParseError::new(
                    ParseErrorKind::EmptyParagraph,
                    rule,
                    extracted,
                ))
            }
        }
    }
}
