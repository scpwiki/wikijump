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
            r#"\[\[\s*date\s+(?P<timestamp>-?[0-9]+)(?:\s+format\s*=\s*"(?P<format>.*)")?\s*\]\]"#
        )
            .case_insensitive(true)
            .build()
            .unwrap()
    };

    static ref EQUATION_REF: Regex = {
        RegexBuilder::new(r"\[\[\s*eref\s+([a-z0-9\-+_\.%]+)\s*\]\]")
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
        RegexBuilder::new(r"\[\[\s*form\s*\]\]\n(?P<contents>.*\n)\[\[/\s*form\s*\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };

    static ref MODULE: Regex = {
        RegexBuilder::new(r"\[\[\s*module[^\]]*\]\]\n(?P<contents>.*\n)\[\[\s*module\s*\]\]")
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
        RegexBuilder::new(r"\[\[\s*(?P<picture>\*)?\s*user\s+(?P<username>[^ ]+)\s*\]\]")
            .case_insensitive(true)
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
        float: bool,
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
        contents: Vec<Line<'a>>,
    },
    Span {
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        contents: Vec<Line<'a>>,
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

        let pair = get_first_pair!(pair);

        macro_rules! as_str {
            () => ( pair.as_str() )
        }

        macro_rules! extract {
            ($regex:expr) => (
                $regex.captures(as_str!())
                    .expect("String doesn't match regular expression")
                    .get(0)
                    .expect("No captures in regular expression")
                    .as_str()
            )
        }

        macro_rules! make_lines {
            () => ( make_lines!(pair) );
            ($pair:expr) => ( $pair.into_inner().map(Line::from_pair).collect() );
        }

        macro_rules! make_words {
            () => ( make_words!(pair) );
            ($pair:expr) => ( $pair.into_inner().map(Word::from_pair).collect() );
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
                let capture = DATE.captures(as_str!()).expect("Regular expression DATE didn't match");

                Word::Date {
                    timestamp: capture["timestamp"].parse().expect("Unable to parse timestamp integer"),
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
                contents: make_lines!(get_first_pair!(pair)),
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
                    .expect("Regular expression MODULE didn't match")
                    .name("contents")
                    .map(|capture| capture.as_str());

                // Parse arguments
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::ident => name = pair.as_str(),
                        Rule::module_arg => {
                            let key = get_nth_pair!(pair, 0).as_str();
                            let value = {
                                let pair = get_nth_pair!(pair, 1);
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
                        Rule::direction => match pair.as_str() {
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
                            _ => panic!("Invalid image alignment: {}", pair.as_str()),
                        },
                        Rule::ident => filename = pair.as_str(),
                        Rule::image_arg => {
                            let capture = ARGUMENT_NAME
                                .captures(pair.as_str())
                                .expect("Regular expression ARGUMENT_NAME didn't match");
                            let name = capture!(capture, "name");
                            let value_pair = get_first_pair!(pair);

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
                }
            }
            Rule::size => {
                let mut contents = Vec::new();

                let mut pairs = pair.into_inner();
                let size = pairs.next().unwrap().as_str();
                let lines = pairs.next().unwrap();

                for pair in lines.into_inner() {
                    contents.push(Line::from_pair(pair));
                }

                Word::Size { size, contents }
            }
            Rule::span => {
                let mut id = None;
                let mut class = None;
                let mut style = None;
                let mut contents = Vec::new();

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::span_arg => {
                            let capture = ARGUMENT_NAME
                                .captures(pair.as_str())
                                .expect("Regular expression ARGUMENT_NAME didn't match");
                            let name = capture!(capture, "name");
                            let value_pair = get_first_pair!(pair);

                            debug_assert_eq!(value_pair.as_rule(), Rule::string);

                            let value = value_pair.as_str();
                            match name {
                                "id" => id = Some(value),
                                "class" => class = Some(value),
                                "style" => style = Some(value),
                                _ => panic!("Unknown argument for [[span]]: {}", name),
                            }
                        }
                        Rule::lines_maybe => contents = make_lines!(pair),
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
                let capture = USER.captures(as_str!()).expect("Regular expression USER didn't match");

                Word::User {
                    username: capture!(capture, "username"),
                    show_picture: capture.name("picture").is_some(),
                }
            }
            _ => panic!("Invalid rule for word: {:?}", pair.as_rule()),
        }
    }
}

#[test]
fn test_regexes() {
    let _ = &*ANCHOR;
    let _ = &*DATE;
    let _ = &*EQUATION_REF;
    let _ = &*FILENAME;
    let _ = &*FORM;
    let _ = &*MODULE;
    let _ = &*RAW;
    let _ = &*USER;
}
