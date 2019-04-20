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

use crate::enums::{Alignment, HeadingLevel, ListStyle};
use std::borrow::Cow;
use std::convert::TryFrom;
use super::prelude::*;

lazy_static! {
    static ref ALIGN: Regex = Regex::new(r"^\[\[(?P<direction><|>|=|==)\]\]").unwrap();
    static ref CLEAR_FLOAT: Regex = Regex::new(r"~{4,}(?P<direction><|>|=|==)?").unwrap();

    static ref CODE_BLOCK: Regex = {
        RegexBuilder::new(r"\[\[\s*code[^\]]*\]\]\n(?P<contents>(?:.*\n)?)\[\[/\s*code\s*\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };

    static ref QUOTE_BLOCK_OLD: Regex = Regex::new(r"^(?P<depth>>+) *(?P<contents>[^\n]*)").unwrap();

    static ref WORDS: Regex = Regex::new(r"^(?P<flag>\+{1,6}|=?)").unwrap();
}

pub fn convert_internal_lines(pair: Pair<Rule>) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut inner = None;
    let mut newlines = 0;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::line => lines.push(Line::from_pair(pair)),
            Rule::line_inner => inner = Some(LineInner::from_pair(pair)),
            Rule::newlines => newlines = pair.as_str().len(),
            _ => panic!("Invalid rule for internal-lines: {:?}", pair.as_rule()),
        }
    }

    if let Some(inner) = inner {
        lines.push(Line { inner, newlines });
    }

    lines
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line<'a> {
    inner: LineInner<'a>,
    newlines: usize,
}

impl<'a> Line<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>) -> Self {
        trace!("Converting pair into Line...");

        let inner;
        let newlines;

        match pair.as_rule() {
            Rule::line_inner => {
                inner = LineInner::from_pair(pair);
                newlines = 0;
            }
            Rule::line => {
                let mut pairs = pair.into_inner();

                inner = {
                    let pair = pairs.next().expect("Line pairs iterator was empty");
                    debug_assert_eq!(pair.as_rule(), Rule::line_inner);
                    LineInner::from_pair(pair)
                };
                newlines = {
                    let pair = pairs
                        .next()
                        .expect("Line pairs iterator only had one element");
                    debug_assert_eq!(pair.as_rule(), Rule::newlines);
                    pair.as_str().len()
                };
            }
            Rule::lines_internal => {
                panic!("The rule 'lines_internal' returns multiple Line instances")
            }
            _ => panic!("Invalid rule for line: {:?}", pair.as_rule()),
        }

        Line { inner, newlines }
    }

    #[inline]
    pub fn inner(&self) -> &LineInner {
        &self.inner
    }

    #[inline]
    pub fn newlines(&self) -> usize {
        self.newlines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineInner<'a> {
    Align {
        alignment: Alignment,
        lines: Vec<Line<'a>>,
    },
    Center {
        words: Vec<Word<'a>>,
    },
    ClearFloat {
        direction: Option<Alignment>,
    },
    CodeBlock {
        language: Option<Cow<'a, str>>,
        contents: &'a str,
    },
    Div {
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        lines: Vec<Line<'a>>,
    },
    Heading {
        level: HeadingLevel,
        words: Vec<Word<'a>>,
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
        lines: Vec<Line<'a>>,
    },
    List {
        style: ListStyle,
        depth: usize,
        items: Vec<Line<'a>>,
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
    TableOfContents {
        // TODO: http://community.wikidot.com/help:toc
    },
    QuoteBlockNew {
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        lines: Vec<Line<'a>>,
    },
    QuoteBlockOld {
        lines: Vec<(usize, &'a str)>,
    },
    Words {
        centered: bool,
        words: Vec<Word<'a>>,
    },
}

impl<'a> LineInner<'a> {
    fn from_pair(pair: Pair<'a, Rule>) -> Self {
        trace!("Converting pair into LineInner...");
        debug_assert_eq!(pair.as_rule(), Rule::line_inner);

        let pair = get_first_pair!(pair);

        macro_rules! extract {
            ($regex:expr) => (
                $regex.captures(pair.as_str())
                    .expect("Pair contents doesn't match regular expression")
                    .get(1)
                    .expect("No captures in regular expression")
                    .as_str()
            )
        }

        match pair.as_rule() {
            Rule::align => {
                let alignment = Alignment::try_from(extract!(ALIGN))
                    .expect("Parsed align block had invalid alignment");
                let lines = pair.into_inner().map(Line::from_pair).collect();

                LineInner::Align {
                    alignment,
                    lines,
                }
            }
            Rule::code => {
                let mut language = None;
                let contents = extract!(CODE_BLOCK);

                // Parse arguments
                let pairs = pair.into_inner()
                    .filter(|pair| pair.as_rule() == Rule::code_arg);

                for pair in pairs {
                    let capture = ARGUMENT_NAME
                        .captures(pair.as_str())
                        .expect("Regular expression ARGUMENT_NAME didn't match");
                    let name = capture!(capture, "name");
                    let value_pair = get_first_pair!(pair);

                    debug_assert_eq!(value_pair.as_rule(), Rule::string);

                    let value = value_pair.as_str();
                    match name {
                        "type" | "lang" | "language" => language = interp_str(value),
                        _ => panic!("Unknown argument for [[code]]: {}", name),
                    }
                }

                LineInner::CodeBlock { language, contents }
            }
            Rule::clear_float => {
                let capture = CLEAR_FLOAT
                    .captures(pair.as_str())
                    .expect("Regular expression CLEAR_FLOAT didn't match");
                let direction = match capture.name("direction") {
                    Some(mtch) => Some(
                        Alignment::try_from(mtch.as_str())
                            .ok()
                            .expect("Alignment conversion failed"),
                    ),
                    None => None,
                };

                LineInner::ClearFloat { direction }
            }
            Rule::div => {
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
                            let name = capture!(capture, "name");
                            let value_pair = get_first_pair!(pair);

                            debug_assert_eq!(value_pair.as_rule(), Rule::string);

                            let value = value_pair.as_str();
                            match name.to_ascii_lowercase().as_str() {
                                "id" => id = Some(value),
                                "class" => class = Some(value),
                                "style" => style = Some(value),
                                _ => panic!("Unknown argument for [[div]]: {}", name),
                            }
                        }
                        Rule::line => lines.push(Line::from_pair(pair)),
                        _ => panic!("Invalid rule for div: {:?}", pair.as_rule()),
                    }
                }

                LineInner::Div {
                    id,
                    class,
                    style,
                    lines,
                }
            }
            Rule::bullet_list | Rule::numbered_list => {
                let depth = {
                    let mut depth = 0;
                    for ch in pair.as_str().chars() {
                        match ch {
                            ' ' => depth += 1,
                            _ => break,
                        }
                    }
                    depth
                };

                let style = match pair.as_rule() {
                    Rule::bullet_list => ListStyle::Bullet,
                    Rule::numbered_list => ListStyle::Numbered,
                    _ => unreachable!(),
                };

                let mut items = Vec::new();
                for pair in pair.into_inner() {
                    debug_assert_eq!(pair.as_rule(), Rule::list_item);

                    let mut words = Vec::new();
                    for pair in pair.into_inner() {
                        words.push(Word::from_pair(pair));
                    }

                    let inner = LineInner::Words { words, centered: false };
                    let line = Line { inner, newlines: 0 };
                    items.push(line);
                }

                LineInner::List { style, depth, items }
            }
            Rule::horizontal_line => LineInner::HorizontalLine,
            Rule::quote_block_new => {
                let mut id = None;
                let mut class = None;
                let mut style = None;
                let mut lines = Vec::new();

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::quote_block_arg => {
                            let capture = ARGUMENT_NAME
                                .captures(pair.as_str())
                                .expect("Regular expression ARGUMENT_NAME didn't match");
                            let name = capture!(capture, "name");
                            let value_pair = get_first_pair!(pair);

                            debug_assert_eq!(value_pair.as_rule(), Rule::string);

                            let value = value_pair.as_str();
                            match name.to_ascii_lowercase().as_str() {
                                "id" => id = Some(value),
                                "class" => class = Some(value),
                                "style" => style = Some(value),
                                _ => panic!("Unknown argument for [[quote]]: {}", name),
                            }
                        }
                        Rule::lines_internal => lines = convert_internal_lines(pair),
                        _ => panic!("Invalid rule for quote: {:?}", pair.as_rule()),
                    }
                }

                LineInner::QuoteBlockNew {
                    id,
                    class,
                    style,
                    lines,
                }
            }
            Rule::quote_block_wikidot => {
                let mut lines = Vec::new();

                for pair in pair.into_inner() {
                    debug_assert_eq!(pair.as_rule(), Rule::quote_block_line);
                    let capture = QUOTE_BLOCK_OLD
                        .captures(pair.as_str())
                        .expect("Regular expression QUOTE_BLOCK_OLD didn't match");
                    let depth = capture["depth"].len();
                    let contents = capture
                        .name("contents")
                        .expect("No match group 'contents' found")
                        .as_str();
                    lines.push((depth, contents));
                }

                LineInner::QuoteBlockOld { lines }
            }
            Rule::words => {
                let flag = extract!(WORDS);
                let mut words = Vec::new();
                for pair in pair.into_inner() {
                    words.push(Word::from_pair(pair));
                }

                match flag {
                    "=" => LineInner::Words {
                        words,
                        centered: true,
                    },
                    "" => LineInner::Words {
                        words,
                        centered: false,
                    },
                    _ => {
                        let level = HeadingLevel::try_from(flag.len())
                            .expect("Regular expression returned incorrectly-sized heading");

                        LineInner::Heading { words, level }
                    }
                }
            }

            _ => panic!("Line rule for {:?} unimplemented!", pair.as_rule()),
            //_ => panic!("Invalid rule for line_inner: {:?}", pair.as_rule()),
        }
    }
}

#[test]
fn test_regexes() {
    let _ = &*ALIGN;
    let _ = &*CLEAR_FLOAT;
    let _ = &*CODE_BLOCK;
    let _ = &*QUOTE_BLOCK_OLD;
    let _ = &*WORDS;
}
