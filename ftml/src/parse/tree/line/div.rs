/*
 * parse/tree/line/div.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

pub fn parse(pair: Pair<Rule>) -> Result<Line> {
    let mut id = None;
    let mut class = None;
    let mut style = None;
    let mut lines = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::div_arg => {
                let capture = ARGUMENT_NAME
                    .captures(pair.as_str())
                    .expect("Regular expression ARGUMENT_NAME didn't match");
                let key = capture!(capture, "name");
                let value_pair = get_first_pair!(pair);

                debug_assert_eq!(value_pair.as_rule(), Rule::string);

                let value = value_pair.as_str();
                match key.to_ascii_lowercase().as_str() {
                    "id" => id = Some(value),
                    "class" => class = Some(value),
                    "style" => style = Some(value),
                    _ => panic!("Unknown argument for [[div]]: {}", key),
                }
            }
            Rule::line => {
                let line = Line::from_pair(pair)?;
                lines.push(line);
            }
            _ => panic!("Invalid rule for div: {:?}", pair.as_rule()),
        }
    }

    Ok(Line::Div {
        id,
        class,
        style,
        lines,
    })
}
