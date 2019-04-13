/*
 * parse/tree/line.rs
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

use crate::enums::{Alignment, ListStyle};
use super::prelude::*;

lazy_static! {
    static ref CLEAR_FLOAT: Regex = Regex::new(r"~{4,}(?P<direction><|>|=|==)?").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line<'a> {
    Align {
        alignment: Alignment,
    },
    Button {
        /*
         https://www.wikidot.com/doc-wiki-syntax:buttons
         btype: ButtonType,
         style: String,
         */
    },
    Center {
        contents: Vec<Word<'a>>,
    },
    ClearFloat {
        direction: Option<Alignment>,
    },
    CodeBlock {
        language: Option<&'a str>,
        contents: Vec<Line<'a>>,
    },
    Div {
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        contents: Vec<Line<'a>>,
    },
    Gallery,
    Heading {
        contents: Vec<Word<'a>>,
    },
    HorizontalLine,
    Html {
        contents: &'a str,
    },
    Iframe {
        url: &'a str,
        args: Option<&'a str>,
    },
    IfTags {
        required: Vec<&'a str>,
        prohibited: Vec<&'a str>,
        contents: Vec<Line<'a>>,
    },
    List {
        style: ListStyle,
        items: Vec<Vec<Word<'a>>>,
    },
    Math {
        label: Option<&'a str>,
        id: Option<&'a str>,
        latex_env: Option<&'a str>,
        expr: &'a str,
    },
    Table {
        rows: Vec<TableRow<'a>>,
    },
    TabView {
        class: Option<&'a str>,
        tabs: Vec<Line<'a>>,
    },
    TableOfContents {
        // TODO: http://community.wikidot.com/help:toc
    },
    Text {
        contents: Vec<Word<'a>>,
    },
}

impl<'a> Line<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>) -> Self {
        trace!("Converting pair into Line...");
        debug_assert_eq!(pair.as_rule(), Rule::line);

        macro_rules! as_str {
            () => ( pair.as_str() )
        }

        let first_pair = pair.clone().into_inner().next().unwrap();
        match first_pair.as_rule() {
            Rule::clear_float => {
                let capture = CLEAR_FLOAT.captures(as_str!()).unwrap();
                let direction = match capture.name("direction") {
                    Some(mtch) => Alignment::from_str(mtch.as_str()),
                    None => None,
                };

                Line::ClearFloat { direction }
            }
            Rule::div => {
                let mut id = None;
                let mut class = None;
                let mut style = None;
                let mut contents = Vec::new();

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::div_arg => {
                            let capture = ARGUMENT_NAME.captures(pair.as_str()).unwrap();
                            let name = capture!(capture, "name");
                            let value_pair = pair.into_inner().next().unwrap();

                            debug_assert_eq!(value_pair.as_rule(), Rule::string);

                            let value = value_pair.as_str();
                            match name {
                                "id" => id = Some(value),
                                "class" => class = Some(value),
                                "style" => style = Some(value),
                                _ => panic!("Unknown argument for [[div]]: {}", name),
                            }
                        }
                        Rule::line => contents.push(Line::from_pair(pair)),
                        _ => panic!("Invalid rule for div: {:?}", pair.as_rule()),
                    }
                }

                Line::Div {
                    id,
                    class,
                    style,
                    contents,
                }
            }
            Rule::horizontal_line => Line::HorizontalLine,
            Rule::center => {
                let mut contents = Vec::new();

                for pair in pair.into_inner() {
                    contents.push(Word::from_pair(pair));
                }

                Line::Center { contents }
            }
            Rule::word => {
                let mut contents = Vec::new();

                for pair in pair.into_inner() {
                    contents.push(Word::from_pair(pair));
                }

                Line::Text { contents }
            }

            _ => unimplemented!(),
            //_ => panic!("Invalid rule for line: {:?}", pair.as_rule()),
        }
    }
}
