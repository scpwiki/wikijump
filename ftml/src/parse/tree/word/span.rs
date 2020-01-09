/*
 * parse/tree/word/span.rs
 *
 * ftml - Convert Wikidot code to HTML
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

pub fn parse(pair: Pair<Rule>) -> Result<Word> {
    let mut id = None;
    let mut class = None;
    let mut style = None;
    let mut paragraphs = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::span_arg => {
                let capture = ARGUMENT_NAME
                    .captures(pair.as_str())
                    .expect("Regular expression ARGUMENT_NAME didn't match");
                let key = capture!(capture, "name");
                let value_pair = get_first_pair!(pair);

                debug_assert_eq!(value_pair.as_rule(), Rule::string);

                let value = interp_str(value_pair.as_str())?;
                match parse_argument(key) {
                    SpanArgument::Id => id = Some(value),
                    SpanArgument::Class => class = Some(value),
                    SpanArgument::Style => style = Some(value),
                }
            }
            Rule::paragraphs_internal => paragraphs = convert_internal_paragraphs(pair)?,
            _ => panic!("Invalid rule for span: {:?}", pair.as_rule()),
        }
    }

    Ok(Word::Span {
        id,
        class,
        style,
        paragraphs,
    })
}

#[derive(Debug, Copy, Clone)]
enum SpanArgument {
    Id,
    Class,
    Style,
}

fn parse_argument(key: &str) -> SpanArgument {
    const SPAN_ARGUMENTS: [(&str, SpanArgument); 3] = [
        ("id", SpanArgument::Id),
        ("class", SpanArgument::Class),
        ("style", SpanArgument::Style),
    ];

    for (name, argument) in &SPAN_ARGUMENTS {
        if key.eq_ignore_ascii_case(name) {
            return *argument;
        }
    }

    panic!("Unknown argument for [[span]]: {}", key);
}
