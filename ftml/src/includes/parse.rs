/*
 * includes/parse.rs
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

use super::IncludeRef;
use crate::data::{PageRef, PageRefParseError};
use crate::log::prelude::*;
use pest::iterators::Pairs;
use pest::Parser;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[grammar = "includes/grammar.pest"]
struct IncludeParser;

pub fn parse_include_block<'t>(
    log: &Logger,
    text: &'t str,
    start: usize,
) -> Result<(IncludeRef<'t>, usize), IncludeParseError> {
    match IncludeParser::parse(Rule::include, text) {
        Ok(mut pairs) => {
            // Extract inner pairs
            // These actually make up the include block's tokens
            let first = pairs.next().expect("No pairs returned on successful parse");
            let span = first.as_span();

            info!(
                log,
                "Parsed include block";
                "span" => SpanWrap::from(span.start()..span.end()),
                "slice" => text,
            );

            // Convert into an IncludeRef
            let include = process_pairs(log, first.into_inner())?;

            // Adjust offset and return
            Ok((include, start + span.end()))
        }
        Err(_error) => {
            warn!(
                log,
                "Include block was invalid";
                "error" => str!(_error),
                "start" => start,
                "slice" => text,
            );

            Err(IncludeParseError)
        }
    }
}

fn process_pairs<'t>(
    log: &Logger,
    mut pairs: Pairs<'t, Rule>,
) -> Result<IncludeRef<'t>, IncludeParseError> {
    let page_raw = pairs.next().ok_or(IncludeParseError)?.as_str();
    let page_ref = PageRef::parse(page_raw)?;

    debug!(
        log,
        "Got page for include";
        "site" => page_ref.site(),
        "page" => page_ref.page(),
    );

    let mut arguments = HashMap::new();
    for pair in pairs {
        debug_assert_eq!(pair.as_rule(), Rule::argument);

        let (key, value) = {
            let mut argument_pairs = pair.into_inner();

            let key = argument_pairs
                .next()
                .expect("Argument pairs terminated early")
                .as_str();

            let value = argument_pairs
                .next()
                .expect("Argument pairs terminated early")
                .as_str();

            (key, value)
        };

        trace!(
            log,
            "Adding argument for include";
            "key" => key,
            "value" => value,
        );

        arguments.insert(Cow::Borrowed(key), Cow::Borrowed(value));
    }

    Ok(IncludeRef::new(page_ref, arguments))
}

#[derive(Debug, PartialEq, Eq)]
pub struct IncludeParseError;

impl From<PageRefParseError> for IncludeParseError {
    #[inline]
    fn from(_: PageRefParseError) -> Self {
        IncludeParseError
    }
}
