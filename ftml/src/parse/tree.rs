/*
 * parse/tree.rs
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

// FIXME to prevent compile spam
#![allow(dead_code)]

use crate::enums::{Alignment, ListStyle};
use pest::iterators::{Pair, Pairs};
use regex::{Regex, RegexBuilder};
use super::Rule;

lazy_static! {
    static ref ANCHOR: Regex = {
        RegexBuilder::new(r"\[\[#\s*([a-z0-9\-+_.%]+)\s*\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };

    static ref ARGUMENTS: Regex = {
        RegexBuilder::new(r#"\s*(?P<argument>\w+)\s*=\s*(?P<value>"(?:[^\\"]|\\[\\"rnt0'])*")"#)
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

    static ref IMAGE: Regex = {
        RegexBuilder::new(r"\[\[\s*image\s+(?P<filename>[^ ]+)\s+(?P<arguments>.+)\s*\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };

    static ref RAW: Regex = {
        RegexBuilder::new(r"[@`]{2}(.+)[@`]{2}")
            .build()
            .unwrap()
    };

    static ref USER: Regex = {
        RegexBuilder::new(r"\[\[(?P<show-picture>\*)?\s*(?P<username>[^ ]+)\s*\]\]")
            .build()
            .unwrap()
    };
}

macro_rules! capture {
    ($capture:expr, $name:expr) => ( $capture.name($name).unwrap().as_str() )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree<'a> {
    paragraphs: Vec<Paragraph<'a>>,
}

impl<'a> SyntaxTree<'a> {
    pub fn from_paragraph_pairs(pairs: Pairs<'a, Rule>) -> Self {
        trace!("Converting pairs into a SyntaxTree...");

        let paragraphs = pairs
            .into_iter()
            .map(|pair| Paragraph::from_pair(pair))
            .collect();

        SyntaxTree { paragraphs }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Paragraph<'a> {
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
        contents: Vec<Paragraph<'a>>,
    },
    Div {
        class: Option<&'a str>,
        style: Option<&'a str>,
    },
    FootnoteBlock,
    Form {
        contents: &'a str, // actually YAML...
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
        contents: Vec<Paragraph<'a>>,
    },
    List {
        style: ListStyle,
        items: Vec<Word<'a>>,
    },
    Math {
        label: Option<&'a str>,
        id: Option<&'a str>,
        latex_env: Option<&'a str>,
        expr: &'a str,
    },
    Module {
        name: &'a str,
        contents: Option<Vec<Paragraph<'a>>>,
    },
    Note {
        contents: Vec<Paragraph<'a>>,
    },
    Table {
        rows: Vec<TableRow<'a>>,
    },
    TabView {
        class: Option<&'a str>,
        tabs: Vec<Paragraph<'a>>,
    },
    TableOfContents {
        // TODO: http://community.wikidot.com/help:toc
    },
    Text {
        contents: Word<'a>,
    },
}

impl<'a> Paragraph<'a> {
    fn from_pair(pair: Pair<'a, Rule>) -> Self {
        trace!("Converting pair into Paragraph...");
        debug_assert_eq!(pair.as_rule(), Rule::paragraph);

        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::word => Paragraph::Text {
                contents: Word::from_pair(inner),
            },
            _ => panic!("Invalid paragraph case"),
        }
    }
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
        contents: Vec<Paragraph<'a>>,
    },
    Image {
        // See https://www.wikidot.com/doc-wiki-syntax:images
        filename: &'a str,
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
    Monospace {
        contents: Vec<Word<'a>>,
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
    fn from_pair(pair: Pair<'a, Rule>) -> Self {
        trace!("Converting pair into Word...");
        debug_assert_eq!(pair.as_rule(), Rule::word);

        let inner = pair.into_inner().next().unwrap();

        macro_rules! as_str {
            () => ( inner.as_str() )
        }

        macro_rules! extract {
            ($regex:expr) => ( $regex.captures(as_str!()).unwrap().get(0).unwrap().as_str() )
        }

        macro_rules! make_paragraphs {
            () => ( inner.into_inner().map(Paragraph::from_pair).collect() )
        }

        macro_rules! make_words {
            () => ( inner.into_inner().map(Word::from_pair).collect() )
        }

        match inner.as_rule() {
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
                contents: make_paragraphs!(),
            },
            Rule::image => {
                let capture = IMAGE.captures(as_str!()).unwrap();

                let filename = capture!(capture, "filename");
                let arguments = capture!(capture, "arguments");

                let mut link = None;
                let mut alt = None;
                let mut title = None;
                let mut width = None;
                let mut height = None;
                let mut style = None;
                let mut class = None;
                let mut size = None;

                for capture in ARGUMENTS.captures_iter(arguments) {
                    let argument = capture!(capture, "argument");
                    let value = capture!(capture, "value");

                    match argument {
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
                        _ => {
                            // For now, ignore unknown arguments
                            warn!(
                                "Ignoring unknown argument in [[image]]: {} = {}",
                                argument, value
                            );
                        }
                    }
                }

                Word::Image {
                    filename,
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
            Rule::span => unimplemented!(),
            Rule::user => {
                let capture = USER.captures(as_str!()).unwrap();

                Word::User {
                    username: capture!(capture, "username"),
                    show_picture: capture.name("show-picture").is_some(),
                }
            }
            _ => panic!("Invalid word case"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow<'a> {
    columns: Vec<Word<'a>>,
    title: bool,
}
