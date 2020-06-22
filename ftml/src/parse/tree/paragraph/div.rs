/*
 * parse/tree/paragraph/div.rs
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

pub fn parse(pair: Pair<Rule>) -> Result<Paragraph> {
    let mut id = None;
    let mut class = None;
    let mut style = None;
    let mut paragraphs = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::div_arg => {
                let capture = ARGUMENT_NAME
                    .captures(pair.as_str())
                    .expect("Regular expression ARGUMENT_NAME didn't match");
                let key = capture!(capture, "name");
                let value_pair = get_first_pair!(pair);

                debug_assert_eq!(value_pair.as_rule(), Rule::string);

                let value = interp_str(value_pair.as_str())?;
                match get_argument(key) {
                    DivArgument::Id => id = Some(value),
                    DivArgument::Class => class = Some(value),
                    DivArgument::Style => style = Some(value),
                }
            }
            Rule::paragraph => {
                let paragraph = Paragraph::from_pair(pair)?;
                paragraphs.push(paragraph);
            }
            _ => panic!("Invalid rule for div: {:?}", pair.as_rule()),
        }
    }

    Ok(Paragraph::Div {
        id,
        class,
        style,
        paragraphs,
    })
}

#[derive(Debug, Copy, Clone)]
enum DivArgument {
    Id,
    Class,
    Style,
}

fn get_argument(key: &str) -> DivArgument {
    const DIV_ARGUMENTS: [(&str, DivArgument); 3] = [
        ("id", DivArgument::Id),
        ("class", DivArgument::Class),
        ("style", DivArgument::Style),
    ];

    for (name, argument) in &DIV_ARGUMENTS {
        if name.eq_ignore_ascii_case(key) {
            return *argument;
        }
    }

    panic!("Unknown argument for [[div]]: {}", key);
}
