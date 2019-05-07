/*
 * parse/tree/word/image.rs
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

use crate::enums::Alignment;
use super::prelude::*;

pub fn parse(pair: Pair<Rule>) -> Result<Word> {
    let mut filename = "";

    let mut float = false;
    let mut direction = None;
    let mut link = None;
    let mut alt = None;
    let mut title = None;
    let mut width = None;
    let mut height = None;
    let mut style = None;
    let mut class = None;
    let mut size = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::image_alignment => match pair.as_str().trim() {
                "f<" => {
                    float = true;
                    direction = Some(Alignment::Left);
                }
                "f>" => {
                    float = true;
                    direction = Some(Alignment::Right);
                }
                "<" => direction = Some(Alignment::Left),
                ">" => direction = Some(Alignment::Right),
                "=" => direction = Some(Alignment::Center),
                "" => direction = None,
                _ => panic!("Invalid image alignment: {}", pair.as_str()),
            },
            Rule::file_ident => filename = pair.as_str(),
            Rule::image_arg => {
                let capture = ARGUMENT_NAME
                    .captures(pair.as_str())
                    .expect("Regular expression ARGUMENT_NAME didn't match");
                let key = capture!(capture, "name");
                let value_pair = get_first_pair!(pair);

                debug_assert_eq!(value_pair.as_rule(), Rule::string);

                let value = value_pair.as_str();
                match key.to_ascii_lowercase().as_str() {
                    "link" => {
                        if value.starts_with("*") {
                            link = Some((&value[1..], true));
                        } else {
                            link = Some((value, false));
                        }
                    }
                    "alt" => alt = Some(value),
                    "title" => title = Some(value),
                    "width" => width = Some(value),
                    "height" => height = Some(value),
                    "style" => style = Some(value),
                    "class" => class = Some(value),
                    "size" => size = Some(value),
                    _ => panic!("Unknown argument for [[image]]: {}", key),
                }
            }
            _ => panic!("Invalid rule for image: {:?}", pair.as_rule()),
        }
    }

    debug_assert_ne!(filename, "", "Filename wasn't produced by parser");

    Ok(Word::Image {
        filename,
        float,
        direction,
        link,
        alt,
        title,
        width,
        height,
        style,
        class,
        size,
    })
}
