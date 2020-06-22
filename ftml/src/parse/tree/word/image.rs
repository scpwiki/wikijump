/*
 * parse/tree/word/image.rs
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
use super::ImageArguments;
use crate::enums::Alignment;

pub fn parse(pair: Pair<Rule>) -> Result<Word> {
    let mut filename = "";

    let mut float = false;
    let mut direction = None;
    let mut link = None;
    let mut arguments = Box::new(ImageArguments::default());

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::image_alignment => {
                let (direction2, float2) = parse_alignment(pair.as_str());
                direction = direction2;
                float = float2;
            }
            Rule::file_ident => filename = pair.as_str(),
            Rule::image_arg => {
                let capture = ARGUMENT_NAME
                    .captures(pair.as_str())
                    .expect("Regular expression ARGUMENT_NAME didn't match");
                let key = capture!(capture, "name");
                let value_pair = get_first_pair!(pair);

                debug_assert_eq!(value_pair.as_rule(), Rule::string);

                let value = value_pair.as_str();
                match parse_image_arg(key) {
                    ImageArgument::Link => {
                        if value.starts_with('*') {
                            link = Some((&value[1..], true));
                        } else {
                            link = Some((value, false));
                        }
                    }
                    ImageArgument::Alt => arguments.alt = interp_str(value).ok(),
                    ImageArgument::Title => arguments.title = interp_str(value).ok(),
                    ImageArgument::Width => arguments.width = interp_str(value).ok(),
                    ImageArgument::Height => arguments.height = interp_str(value).ok(),
                    ImageArgument::Style => arguments.style = interp_str(value).ok(),
                    ImageArgument::Class => arguments.class = interp_str(value).ok(),
                    ImageArgument::Size => arguments.size = interp_str(value).ok(),
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
        arguments,
    })
}

fn parse_alignment(key: &str) -> (Option<Alignment>, bool) {
    match key.trim() {
        "f<" => (Some(Alignment::Left), true),
        "f>" => (Some(Alignment::Right), true),
        "<" => (Some(Alignment::Left), false),
        ">" => (Some(Alignment::Right), false),
        "=" => (Some(Alignment::Center), false),
        "" => (None, false),
        _ => panic!("Invalid image alignment: {}", key),
    }
}

#[derive(Debug, Copy, Clone)]
enum ImageArgument {
    Link,
    Alt,
    Title,
    Width,
    Height,
    Style,
    Class,
    Size,
}

fn parse_image_arg(key: &str) -> ImageArgument {
    const IMAGE_ARGUMENT_VALUES: [(&str, ImageArgument); 8] = [
        ("link", ImageArgument::Link),
        ("alt", ImageArgument::Alt),
        ("title", ImageArgument::Title),
        ("width", ImageArgument::Width),
        ("height", ImageArgument::Height),
        ("style", ImageArgument::Style),
        ("class", ImageArgument::Class),
        ("size", ImageArgument::Size),
    ];

    for (name, argument) in &IMAGE_ARGUMENT_VALUES {
        if key.eq_ignore_ascii_case(name) {
            return *argument;
        }
    }

    panic!("Unknown argument for [[image]]: {}", key);
}
