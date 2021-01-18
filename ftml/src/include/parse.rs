/*
 * include/parse.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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

use super::{IncludeRef, PageRef};
use crate::span_wrap::SpanWrap;
use pest::iterators::Pairs;
use pest::Parser;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Parser, Debug)]
#[grammar = "include/grammar.pest"]
struct IncludeParser;

pub fn parse_include_block<'t>(
    log: &slog::Logger,
    text: &'t str,
    range: Range<usize>,
) -> Option<IncludeRef<'t>> {
    match IncludeParser::parse(Rule::include, text) {
        Ok(mut pairs) => {
            debug!(
                log,
                "Parsed include block";
                "span" => SpanWrap::from(range),
                "slice" => text,
            );

            // Extract inner pairs
            // These actually make up the include block's tokens
            let inner_pairs = pairs
                .next()
                .expect("No pairs returned on successful parse")
                .into_inner();

            // Convert into an IncludeRef
            process_pairs(log, inner_pairs)
        }
        Err(error) => {
            debug!(
                log,
                "Include block was invalid";
                "error" => str!(error),
                "span" => SpanWrap::from(range),
                "slice" => text,
            );

            None
        }
    }
}

fn process_pairs<'t>(
    log: &slog::Logger,
    mut pairs: Pairs<'t, Rule>,
) -> Option<IncludeRef<'t>> {
    let page_raw = match pairs.next() {
        Some(pair) => pair.as_str(),
        None => return None,
    };

    let page_ref = match PageRef::parse(page_raw) {
        Some(page_ref) => page_ref,
        None => return None,
    };

    trace!(
        log, "Got page for include"; "site" => page_ref.site(), "page" => page_ref.page(),
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

    Some(IncludeRef::new(page_ref, arguments))
}
