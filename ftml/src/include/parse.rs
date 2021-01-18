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
use std::ops::Range;

#[derive(Parser, Debug)]
#[grammar = "include/grammar.pest"]
struct IncludeParser;

pub fn parse_include_block<'t>(
    log: &slog::Logger,
    text: &'t str,
    range: Range<usize>,
) -> Result<IncludeRef<'t>, ()> {
    match IncludeParser::parse(Rule::include, text) {
        Ok(mut pairs) => {
            debug!(
                log,
                "Parsed include block";
                "span" => SpanWrap::from(range),
                "slice" => text,
            );

            process_pairs(pairs)
        }
        Err(error) => {
            debug!(
                log,
                "Found invalid include block";
                "error" => str!(error),
                "span" => SpanWrap::from(range),
                "slice" => text,
            );

            Err(())
        }
    }
}

fn process_pairs<'t>(mut pairs: Pairs<'t, Rule>) -> Result<IncludeRef<'t>, ()> {
    let page_raw = match pairs.next() {
        Some(pair) => pair.as_str(),
        None => return Err(()),
    };

    for pair in pairs {
        // TODO
        println!("rule: {:?}, slice: {:?}", pair.as_rule(), pair.as_str());
    }

    todo!()
}
