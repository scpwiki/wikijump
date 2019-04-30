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

use crate::Result;
use crate::enums::{Alignment, AnchorTarget};
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;
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
        RegexBuilder::new(r"\[\[\s*form\s*\]\]\n(?P<contents>(?:.*\n)?)\[\[/\s*form\s*\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };

    static ref MODULE: Regex = {
        RegexBuilder::new(r"\[\[\s*module\s+[^\]]*\]\](?:\n(?P<contents>.*\n)\[\[\s*module\s*\]\])?")
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
        href: Option<&'a str>,
        name: Option<&'a str>,
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        target: Option<AnchorTarget>,
        words: Vec<Word<'a>>,
    },
    Bold {
        words: Vec<Word<'a>>,
    },
    Button {
        /*
         https://www.wikidot.com/doc-wiki-syntax:buttons
         btype: ButtonType,
         style: String,
         */
    },
    Collapsible {
        show_top: bool,
        show_bottom: bool,
        lines: Vec<Line<'a>>,
    },
    Color {
        color: &'a str,
        words: Vec<Word<'a>>,
    },
    Date {
        timestamp: i64,
        format: Option<&'a str>,
    },
    Email {
        address: &'a str,
        text: Option<&'a str>,
    },
    EquationReference {
        name: &'a str,
    },
    File {
        filename: &'a str,
    },
    Footnote {
        lines: Vec<Line<'a>>,
    },
    FootnoteBlock,
    Form {
        contents: &'a str, // actually YAML...
    },
    Gallery,
    Image {
        // See https://www.wikidot.com/doc-wiki-syntax:images
        filename: &'a str,
        float: bool,
        direction: Option<Alignment>,
        link: Option<(&'a str, bool)>,
        alt: Option<&'a str>,
        title: Option<&'a str>,
        width: Option<&'a str>,
        height: Option<&'a str>,
        style: Option<&'a str>,
        class: Option<&'a str>,
        size: Option<&'a str>,
    },
    Italics {
        words: Vec<Word<'a>>,
    },
    Link {
        href: &'a str,
        target: Option<AnchorTarget>,
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
        words: Vec<Word<'a>>,
    },
    Note {
        lines: Vec<Line<'a>>,
    },
    Raw {
        contents: &'a str,
    },
    Size {
        size: &'a str,
        lines: Vec<Line<'a>>,
    },
    Span {
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        lines: Vec<Line<'a>>,
    },
    Strikethrough {
        words: Vec<Word<'a>>,
    },
    Subscript {
        words: Vec<Word<'a>>,
    },
    Superscript {
        words: Vec<Word<'a>>,
    },
    TabList {
        tabs: Vec<Tab<'a>>,
    },
    Text {
        contents: &'a str,
    },
    Underline {
        words: Vec<Word<'a>>,
    },
    User {
        username: &'a str,
        show_picture: bool,
    },
}

impl<'a> Word<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>) -> Result<Self> {
        trace!("Converting pair into Word...");
        debug_assert_eq!(pair.as_rule(), Rule::word);

        let pair = get_first_pair!(pair);

        macro_rules! extract {
            ($regex:expr) => ( extract!($regex, pair) );
            ($regex:expr, $pair:expr) => (
                $regex.captures($pair.as_str())
                    .expect("Pair contents doesn't match regular expression")
                    .get(0)
                    .expect("No captures in regular expression")
                    .as_str()
            );
        }

        macro_rules! make_words {
            () => ( make_words!(pair) );
            ($pair:expr) => {{
                let word_res: Result<Vec<_>> = $pair
                    .into_inner()
                    .map(Word::from_pair)
                    .collect();

                word_res?
            }};
        }

        let word = match pair.as_rule() {
            Rule::text => Word::Text {
                contents: pair.as_str(),
            },
            Rule::raw | Rule::legacy_raw => Word::Raw {
                contents: extract!(RAW),
            },
            Rule::email => Word::Email {
                address: pair.as_str(),
                text: None,
            },
            Rule::em_dash => {
                // \u{2014} is an em dash: '—'
                Word::Text { contents: "\u{2014}" }
            }
            Rule::color => {
                let mut color = "";
                let mut words = Vec::new();

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::ident => color = pair.as_str(),
                        Rule::word => {
                            let word = Word::from_pair(pair)?;
                            words.push(word);
                        }
                        _ => panic!("Invalid rule for color: {:?}", pair.as_rule()),
                    }
                }

                Word::Color { color, words }
            }
            Rule::italics => Word::Italics {
                words: make_words!(),
            },
            Rule::strikethrough => Word::Strikethrough {
                words: make_words!(),
            },
            Rule::bold => Word::Bold {
                words: make_words!(),
            },
            Rule::underline => Word::Underline {
                words: make_words!(),
            },
            Rule::subscript => Word::Subscript {
                words: make_words!(),
            },
            Rule::superscript => Word::Superscript {
                words: make_words!(),
            },
            Rule::monospace => Word::Monospace {
                words: make_words!(),
            },
            Rule::anchor => Word::Anchor {
                href: None,
                name: Some(extract!(ANCHOR)),
                id: None,
                class: None,
                style: None,
                target: None,
                words: Vec::new(),
            },
            Rule::anchor_tag => {
                let mut href = None;
                let mut name = None;
                let mut id = None;
                let mut class = None;
                let mut style = None;
                let mut target = None;
                let mut words = Vec::new();

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::anchor_arg => {
                            let capture = ARGUMENT_NAME
                                .captures(pair.as_str())
                                .expect("Regular expression ARGUMENT_NAME didn't match");
                            let key = capture!(capture, "name");
                            let value_pair = get_first_pair!(pair);

                            debug_assert_eq!(value_pair.as_rule(), Rule::string);

                            let value = value_pair.as_str();
                            match key.to_ascii_lowercase().as_str() {
                                "href" => href = Some(value),
                                "name" => name = Some(value),
                                "id" => id = Some(value),
                                "class" => class = Some(value),
                                "style" => style = Some(value),
                                "target" => target = AnchorTarget::try_from(value).ok(),
                                _ => panic!("Unknown argument for [[a]]: {}", key),
                            }
                        }
                        Rule::word => {
                            let word = Word::from_pair(pair)?;
                            words.push(word);
                        }
                        _ => panic!("Invalid rule for anchor: {:?}", pair.as_rule()),
                    }
                }

                Word::Anchor { href, name, id, class, target, style, words }
            }
            Rule::date => {
                let capture = DATE.captures(pair.as_str())
                    .expect("Regular expression DATE didn't match");

                Word::Date {
                    timestamp: capture["timestamp"]
                        .parse()
                        .expect("Unable to parse timestamp integer"),
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
                lines: convert_internal_lines(get_first_pair!(pair))?,
            },
            Rule::footnote_block => Word::FootnoteBlock,
            Rule::form => Word::Form {
                contents: extract!(FORM),
            },
            Rule::gallery => Word::Gallery,
            Rule::module => {
                let mut name = "";
                let mut arguments = HashMap::new();

                let contents = MODULE
                    .captures(pair.as_str())
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
                let mut lines = Vec::new();

                for pair in pair.into_inner() {
                    let line = Line::from_pair(pair)?;
                    lines.push(line);
                }

                Word::Note { lines }
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
            Rule::link_bare => {
                let mut pairs = pair.into_inner();

                let target = get_link_target(
                    pairs.next().expect("LinkBare pairs iterator was empty")
                );

                let href = pairs
                    .next()
                    .expect("LinkBare pairs iterator had only one element")
                    .as_str();


                Word::Link { href, target, text: None }
            }
            Rule::link_page => {
                let mut pairs = pair.into_inner();

                let target = get_link_target(
                    pairs.next().expect("LinkPage pairs iterator was empty")
                );

                let href = pairs
                    .next()
                    .expect("LinkPage pairs iterator had only one element")
                    .as_str();

                let use_page_title = pairs.next().is_some();
                let text = if use_page_title {
                    pairs.next().map(|pair| pair.as_str())
                } else {
                    Some("")
                };

                Word::Link { href, target, text }
            }
            Rule::link_url => {
                let mut pairs = pair.into_inner();

                let target = get_link_target(
                    pairs.next().expect("LinkUrl pairs iterator was empty")
                );

                let href = pairs.next().expect("LinkUrl pairs iterator had only one element");

                let text = pairs
                    .next()
                    .expect("LinkUrl pairs iterator had only two elements")
                    .as_str();
                let text = Some(text);

                match href.as_rule() {
                    Rule::email => Word::Email { address: href.as_str(), text },
                    Rule::link_url_href => Word::Link { href: href.as_str(), target, text },
                    _ => panic!("Invalid rule for link_url: {:?}", href.as_rule()),
                }
            }
            Rule::size => {
                let mut pairs = pair.into_inner();
                let size = {
                    let pair = pairs.next().expect("Size pairs iterator was empty");
                    pair.as_str()
                };
                let lines = {
                    let pair = pairs
                        .next()
                        .expect("Size pairs iterator had only one element");

                    convert_internal_lines(pair)?
                };

                Word::Size { size, lines }
            }
            Rule::span => {
                let mut id = None;
                let mut class = None;
                let mut style = None;
                let mut lines = Vec::new();

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::span_arg => {
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
                                _ => panic!("Unknown argument for [[span]]: {}", key),
                            }
                        }
                        Rule::lines_internal => lines = convert_internal_lines(pair)?,
                        _ => panic!("Invalid rule for span: {:?}", pair.as_rule()),
                    }
                }

                Word::Span {
                    id,
                    class,
                    style,
                    lines,
                }
            }
            Rule::tab_list => {
                let mut tabs = Vec::new();

                // Iterate over tabs
                for pair in pair.into_inner() {
                    let mut pairs = pair.into_inner();
                    let name = {
                        let pair = pairs.next().expect("Tab pairs iterator was empty");
                        pair.as_str()
                    };
                    let contents = {
                        let pair = pairs
                            .next()
                            .expect("Tab pairs iterator had only one element");

                        convert_internal_lines(pair)?
                    };

                    tabs.push(Tab { name, contents });
                }

                Word::TabList { tabs }
            }
            Rule::user => {
                let capture = USER.captures(pair.as_str())
                    .expect("Regular expression USER didn't match");

                Word::User {
                    username: capture!(capture, "username"),
                    show_picture: capture.name("picture").is_some(),
                }
            }
            _ => panic!("Invalid rule for word: {:?}", pair.as_rule()),
        };

        Ok(word)
    }
}

impl<'a> AsRef<Word<'a>> for Word<'a> {
    #[inline]
    fn as_ref(&self) -> &Word<'a> {
        self
    }
}

fn get_link_target(pair: Pair<Rule>) -> Option<AnchorTarget> {
    debug_assert_eq!(pair.as_rule(), Rule::link_newtab);

    match pair.as_str() {
        "*" => Some(AnchorTarget::NewTab),
        "" => None,
        value => panic!("Invalid value for Rule::link_newtab: {:?}", value),
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
