/*
 * parse/tree/line/mod.rs
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

macro_rules! extract {
    ($regex:expr, $pair:expr) => (
        $regex.captures($pair.as_str())
            .expect("Pair contents doesn't match regular expression")
            .get(1)
            .expect("No captures in regular expression")
            .as_str()
    )
}

mod align;
mod code;
mod collapsible;

mod prelude {
    pub use crate::{Error, Result};
    pub use std::borrow::Cow;
    pub use std::convert::TryFrom;
    pub use super::convert_internal_lines;
    pub use super::super::prelude::*;
}

use crate::enums::{Alignment, HeadingLevel, ListStyle};
use self::prelude::*;

lazy_static! {
    static ref CLEAR_FLOAT: Regex = Regex::new(r"~{4,}(?P<direction><|>|=)?").unwrap();

    static ref JAVASCRIPT_BLOCK: Regex = {
        RegexBuilder::new(r"\[\[\s*(?:js|javascript)\s*\]\]\n(?P<contents>(?:.*\n)?)\[\[/\s*(?:js|javascript)\s*\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };

    static ref QUOTE_BLOCK_OLD: Regex = Regex::new(r"^(?P<depth>>+) *(?P<contents>[^\n]*)").unwrap();

    static ref WORDS: Regex = Regex::new(r"^(?P<flag>\+{1,6}|=?)").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line<'a> {
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
    Collapsible {
        show_text: Option<Cow<'a, str>>,
        hide_text: Option<Cow<'a, str>>,
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        show_top: bool,
        show_bottom: bool,
        lines: Vec<Line<'a>>,
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
    Javascript {
        contents: &'a str,
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
    Newlines {
        count: usize,
    },
    Table {
        rows: Vec<TableRow<'a>>,
    },
    TableOfContents {
        // TODO: http://community.wikidot.com/help:toc
    },
    QuoteBlock {
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        lines: Vec<Line<'a>>,
    },
    Words {
        centered: bool,
        words: Vec<Word<'a>>,
    },
}

impl<'a> Line<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>) -> Result<Self> {
        trace!("Converting pair into Line...");

        // Handle outer wrapping
        let pair = match pair.as_rule() {
            Rule::line => get_first_pair!(pair),
            Rule::line_inner => pair,
            Rule::lines_internal => {
                // This indicates a bug in the grammar
                panic!("The rule 'lines_internal' returns multiple Line instances")
            }
            _ => {
                return Err(Error::Msg(format!(
                    "Invalid rule for line: {:?}",
                    pair.as_rule()
                )))
            }
        };

        match pair.as_rule() {
            Rule::line_inner => Self::from_rule_inner(pair),
            Rule::just_newlines => Ok(Line::Newlines {
                count: pair.as_str().len(),
            }),
            _ => panic!("Invalid rule for line: {:?}", pair.as_rule()),
        }
    }

    fn from_rule_inner(pair: Pair<'a, Rule>) -> Result<Self> {
        debug_assert_eq!(pair.as_rule(), Rule::line_inner);
        let pair = get_first_pair!(pair);

        let line_inner = match pair.as_rule() {
            Rule::align => align::parse(pair)?,
            Rule::code => code::parse(pair)?,
            Rule::collapsible => collapsible::parse(pair)?,
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

                Line::ClearFloat { direction }
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

                Line::Div {
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
                        let word = Word::from_pair(pair)?;
                        words.push(word);
                    }

                    let line = Line::Words {
                        words,
                        centered: false,
                    };
                    items.push(line);
                }

                Line::List {
                    style,
                    depth,
                    items,
                }
            }
            Rule::horizontal_line => Line::HorizontalLine,
            Rule::javascript => Line::Javascript { contents: extract!(JAVASCRIPT_BLOCK, pair) },
            Rule::quote_block => {
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
                            let key = capture!(capture, "name");
                            let value_pair = get_first_pair!(pair);

                            debug_assert_eq!(value_pair.as_rule(), Rule::string);

                            let value = value_pair.as_str();
                            match key.to_ascii_lowercase().as_str() {
                                "id" => id = Some(value),
                                "class" => class = Some(value),
                                "style" => style = Some(value),
                                _ => panic!("Unknown argument for [[quote]]: {}", key),
                            }
                        }
                        Rule::lines_internal => lines = convert_internal_lines(pair)?,
                        _ => panic!("Invalid rule for quote: {:?}", pair.as_rule()),
                    }
                }

                Line::QuoteBlock {
                    id,
                    class,
                    style,
                    lines,
                }
            }
            Rule::words => {
                let flag = extract!(WORDS, pair);

                let mut words = Vec::new();
                for pair in pair.into_inner() {
                    let word = Word::from_pair(pair)?;
                    words.push(word);
                }

                match flag {
                    "=" => Line::Words {
                        words,
                        centered: true,
                    },
                    "" => Line::Words {
                        words,
                        centered: false,
                    },
                    _ => {
                        let level = HeadingLevel::try_from(flag.len())
                            .expect("Regular expression returned incorrectly-sized heading");

                        Line::Heading { words, level }
                    }
                }
            }

            _ => {
                return Err(Error::Msg(format!(
                    "Line rule for {:?} unimplemented!",
                    pair.as_rule()
                )))
            }
            //_ => Err(Error::Msg(format!("Invalid rule for line_inner: {:?}", pair.as_rule()))),
        };

        Ok(line_inner)
    }
}

impl<'a> AsRef<Line<'a>> for Line<'a> {
    #[inline]
    fn as_ref(&self) -> &Line<'a> {
        self
    }
}

pub fn convert_internal_lines(pair: Pair<Rule>) -> Result<Vec<Line>> {
    let mut lines = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::line | Rule::line_inner => {
                let line = Line::from_pair(pair)?;
                lines.push(line);
            }
            _ => panic!("Invalid rule for internal-lines: {:?}", pair.as_rule()),
        }
    }

    Ok(lines)
}

#[test]
fn test_regexes() {
    let _ = &*CLEAR_FLOAT;
    let _ = &*QUOTE_BLOCK_OLD;
    let _ = &*WORDS;
}
