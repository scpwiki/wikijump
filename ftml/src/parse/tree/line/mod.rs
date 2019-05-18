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
mod clear_float;
mod code;
mod collapsible;
mod div;
mod iframe;
mod list;
mod quote;
mod words;

mod prelude {
    pub use crate::{Error, Result};
    pub use std::borrow::Cow;
    pub use std::convert::TryFrom;
    pub use super::convert_internal_lines;
    pub use super::super::prelude::*;
}

use crate::enums::{Alignment, HeadingLevel, ListStyle};
use self::prelude::*;
use std::collections::HashMap;

lazy_static! {
    static ref HTML_BLOCK: Regex = {
        RegexBuilder::new(r"\[\[\s*html\s*\]\]\n(?P<contents>(?:.*\n)?)\[\[/\s*html\s*\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };

    static ref JAVASCRIPT_BLOCK: Regex = {
        RegexBuilder::new(r"\[\[\s*(?:js|javascript)\s*\]\]\n(?P<contents>(?:.*\n)?)\[\[/\s*(?:js|javascript)\s*\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
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
        arguments: HashMap<&'a str, Cow<'a, str>>,
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
            Rule::clear_float => clear_float::parse(pair),
            Rule::div => div::parse(pair)?,
            Rule::bullet_list | Rule::numbered_list => list::parse(pair)?,
            Rule::horizontal_line => Line::HorizontalLine,
            Rule::html => Line::Html { contents: extract!(HTML_BLOCK, pair) },
            Rule::iframe => iframe::parse(pair),
            Rule::javascript => Line::Javascript { contents: extract!(JAVASCRIPT_BLOCK, pair) },
            Rule::quote_block => quote::parse(pair)?,
            Rule::words => words::parse(pair)?,

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
    let _ = &*HTML_BLOCK;
    let _ = &*JAVASCRIPT_BLOCK;
}
