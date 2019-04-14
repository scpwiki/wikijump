/*
 * parse/tree/word.rs
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
use std::borrow::Cow;
use std::collections::HashMap;
use super::prelude::*;

lazy_static! {
    static ref ANCHOR: Regex = {
        RegexBuilder::new(r"\[\[#\s*([a-z0-9\-+_.%]+)\s*\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };

    static ref DATE: Regex = {
        RegexBuilder::new(
            r#"\[\[\s*date\s+(?P<timestamp>-?[0-9]+)\s+(?:format\s*=\s*"(?P<format>.*)"\s*\]\]"#
        )
            .case_insensitive(true)
            .build()
            .unwrap()
    };

    static ref EQUATION_REF: Regex = {
        RegexBuilder::new(r"\[\[\s*eref\s+([a-z0-9\-+)\s*\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };

    static ref FILENAME: Regex = {
        RegexBuilder::new(r"\[\[\s*file\s+(.+)\s*\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };

    static ref FORM: Regex = {
        RegexBuilder::new(r"\[\[\s*form\s*\]\]\n(?P<contents>.*)\n\[\[/\s*form\s*\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };

    static ref MODULE: Regex = {
        RegexBuilder::new(r"\[\[\s*module[^\]]*\]\](?:\n(?P<contents>.*)\n\[\[\s*module\s*\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };

    static ref RAW: Regex = {
        RegexBuilder::new(r"[@`]{2}(.*)[@`]{2}")
            .build()
            .unwrap()
    };

    static ref USER: Regex = {
        RegexBuilder::new(r"\[\[(?P<show-picture>\*)?\s*(?P<username>[^ ]+)\s*\]\]")
            .build()
            .unwrap()
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Word<'a> {
    Anchor {
        name: &'a str,
    },
    Bold {
        contents: Vec<Word<'a>>,
    },
    Color {
        color: &'a str,
    },
    Date {
        timestamp: i64,
        format: Option<&'a str>,
    },
    Email {
        contents: &'a str,
    },
    EquationReference {
        name: &'a str,
    },
    File {
        filename: &'a str,
    },
    Footnote {
        contents: Vec<Line<'a>>,
    },
    FootnoteBlock,
    Form {
        contents: &'a str, // actually YAML...
    },
    Image {
        // See https://www.wikidot.com/doc-wiki-syntax:images
        filename: &'a str,
        direction: Option<Alignment>,
        link: Option<(&'a str, bool)>,
        alt: Option<&'a str>,
        title: Option<Cow<'a, str>>,
        width: Option<&'a str>,
        height: Option<&'a str>,
        style: Option<&'a str>,
        class: Option<&'a str>,
        size: Option<&'a str>,
    },
    Italics {
        contents: Vec<Word<'a>>,
    },
    Link {
        page: &'a str,
        anchor: Option<&'a str>,
        text: Option<&'a str>,
    },
    Math {
        expr: &'a str,
    },
    Module {
        name: &'a str,
        arguments: HashMap<&'a str, Cow<'a, str>>,
        contents: Option<&'a str>,
    },
    Monospace {
        contents: Vec<Word<'a>>,
    },
    Note {
        contents: Vec<Line<'a>>,
    },
    Raw {
        contents: &'a str,
    },
    Size {
        size: &'a str,
        contents: Vec<Word<'a>>,
    },
    Span {
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        contents: Vec<Word<'a>>,
    },
    Strikethrough {
        contents: Vec<Word<'a>>,
    },
    Subscript {
        contents: Vec<Word<'a>>,
    },
    Superscript {
        contents: Vec<Word<'a>>,
    },
    Text {
        contents: &'a str,
    },
    Underline {
        contents: Vec<Word<'a>>,
    },
    Url {
        contents: &'a str,
    },
    User {
        username: &'a str,
        show_picture: bool,
    },
}

impl<'a> Word<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>) -> Self {
        trace!("Converting pair into Word...");
        debug_assert_eq!(pair.as_rule(), Rule::word);

        let pair = pair.into_inner().next().unwrap();

        macro_rules! as_str {
            () => ( pair.as_str() )
        }

        macro_rules! extract {
            ($regex:expr) => ( $regex.captures(as_str!()).unwrap().get(0).unwrap().as_str() )
        }

        macro_rules! make_lines {
            () => ( pair.into_inner().map(Line::from_pair).collect() )
        }

        macro_rules! make_words {
            () => ( pair.into_inner().map(Word::from_pair).collect() )
        }

        match pair.as_rule() {
            Rule::text => Word::Text {
                contents: as_str!(),
            },
            Rule::raw | Rule::legacy_raw => Word::Raw {
                contents: extract!(RAW),
            },
            Rule::email => Word::Email {
                contents: as_str!(),
            },
            Rule::italics => Word::Italics {
                contents: make_words!(),
            },
            Rule::strikethrough => Word::Strikethrough {
                contents: make_words!(),
            },
            Rule::bold => Word::Bold {
                contents: make_words!(),
            },
            Rule::underline => Word::Underline {
                contents: make_words!(),
            },
            Rule::subscript => Word::Subscript {
                contents: make_words!(),
            },
            Rule::superscript => Word::Superscript {
                contents: make_words!(),
            },
            Rule::monospace => Word::Monospace {
                contents: make_words!(),
            },
            Rule::anchor => Word::Anchor {
                name: extract!(ANCHOR),
            },
            Rule::date => {
                let capture = DATE.captures(as_str!()).unwrap();

                Word::Date {
                    timestamp: capture["timestamp"].parse().unwrap(),
                    format: capture.name("format").map(|mtch| mtch.as_str()),
                }
            }
            Rule::equation_ref => Word::EquationReference {
                name: extract!(EQUATION_REF),
            },
            Rule::file_ref => Word::File {
                filename: extract!(FILENAME),
            },
            Rule::footnote => Word::Footnote {
                contents: make_lines!(),
            },
            Rule::footnote_block => Word::FootnoteBlock,
            Rule::form => Word::Form {
                contents: extract!(FORM),
            },
            Rule::module => {
                let mut name = "";
                let mut arguments = HashMap::new();

                let contents = MODULE
                    .captures(as_str!())
                    .unwrap()
                    .name("contents")
                    .map(|capture| capture.as_str());

                // Parse arguments
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::ident => name = pair.as_str(),
                        Rule::module_arg => {
                            let key = {
                                let pair = pair.clone().into_inner().nth(0).unwrap();
                                pair.as_str()
                            };

                            let value = {
                                let pair = pair.clone().into_inner().nth(1).unwrap();
                                interp_str(pair.as_str()).expect("Invalid string value")
                            };

                            arguments.insert(key, value);
                        }
                        _ => panic!("Invalid rule for module: {:?}", pair.as_rule()),
                    }
                }

                debug_assert_ne!(name, "", "Module name never set");

                Word::Module {
                    name,
                    arguments,
                    contents,
                }
            }
            Rule::note => {
                let mut contents = Vec::new();

                for pair in pair.into_inner() {
                    contents.push(Line::from_pair(pair));
                }

                Word::Note { contents }
            }
            Rule::image => {
                let mut filename = "";

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
                        Rule::direction => direction = Alignment::from_str(pair.as_str()),
                        Rule::ident => filename = pair.as_str(),
                        Rule::image_arg => {
                            let capture = ARGUMENT_NAME.captures(pair.as_str()).unwrap();
                            let name = capture!(capture, "name");
                            let value_pair = pair.into_inner().next().unwrap();

                            debug_assert_eq!(value_pair.as_rule(), Rule::string);

                            let value = value_pair.as_str();
                            match name {
                                "link" => {
                                    if value.starts_with("*") {
                                        link = Some((&value[1..], true));
                                    } else {
                                        link = Some((value, false));
                                    }
                                }
                                "alt" => alt = Some(value),
                                "title" => title = interp_str(value),
                                "width" => width = Some(value),
                                "height" => height = Some(value),
                                "style" => style = Some(value),
                                "class" => class = Some(value),
                                "size" => size = Some(value),
                                _ => panic!("Unknown argument for [[image]]: {}", name),
                            }
                        }
                        _ => panic!("Invalid rule for image: {:?}", pair.as_rule()),
                    }
                }

                debug_assert_ne!(filename, "", "Filename wasn't produced by parser");

                Word::Image {
                    filename,
                    direction,
                    link,
                    alt,
                    title,
                    width,
                    height,
                    style,
                    class,
                    size,
                }
            }
            Rule::span => {
                let mut id = None;
                let mut class = None;
                let mut style = None;
                let mut contents = Vec::new();

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::span_arg => {
                            let capture = ARGUMENT_NAME.captures(pair.as_str()).unwrap();
                            let name = capture!(capture, "name");
                            let value_pair = pair.into_inner().next().unwrap();

                            debug_assert_eq!(value_pair.as_rule(), Rule::string);

                            let value = value_pair.as_str();
                            match name {
                                "id" => id = Some(value),
                                "class" => class = Some(value),
                                "style" => style = Some(value),
                                _ => panic!("Unknown argument for [[span]]: {}", name),
                            }
                        }
                        Rule::word => contents.push(Word::from_pair(pair)),
                        _ => panic!("Invalid rule for span: {:?}", pair.as_rule()),
                    }
                }

                Word::Span {
                    id,
                    class,
                    style,
                    contents,
                }
            }
            Rule::user => {
                let capture = USER.captures(as_str!()).unwrap();

                Word::User {
                    username: capture!(capture, "username"),
                    show_picture: capture.name("show-picture").is_some(),
                }
            }
            _ => panic!("Invalid rule for word: {:?}", pair.as_rule()),
        }
    }
}
