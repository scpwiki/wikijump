/*
 * parse/tree/paragraph/mod.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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
    ($regex:expr, $pair:expr) => {
        $regex
            .captures($pair.as_str())
            .expect("Pair contents doesn't match regular expression")
            .get(1)
            .expect("No captures in regular expression")
            .as_str()
    };
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
    pub use super::super::prelude::*;
    pub use super::convert_internal_paragraphs;
    pub use crate::{Error, Result};
    pub use std::borrow::Cow;
    pub use std::convert::TryFrom;
}

use self::prelude::*;
use crate::enums::{Alignment, HeadingLevel, ListStyle};
use std::collections::HashMap;

lazy_static! {
    static ref HTML_BLOCK: Regex = {
        RegexBuilder::new(
            r"(?x)
            \[\[\s*html\s*\]\]\n
            (?P<contents>(?:.*\n)?)
            \[\[/\s*html\s*\]\]",
        )
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
    };
    static ref JAVASCRIPT_BLOCK: Regex = {
        RegexBuilder::new(
            r"(?x)
            \[\[\s*(?:js|javascript)\s*\]\]\n
            (?P<contents>(?:.*\n)?)
            \[\[/\s*(?:js|javascript)\s*\]\]",
        )
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Paragraph<'a> {
    Align {
        alignment: Alignment,
        paragraphs: Vec<Paragraph<'a>>,
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
        id: Option<Cow<'a, str>>,
        class: Option<Cow<'a, str>>,
        style: Option<Cow<'a, str>>,
        show_top: bool,
        show_bottom: bool,
        paragraphs: Vec<Paragraph<'a>>,
    },
    Div {
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        paragraphs: Vec<Paragraph<'a>>,
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
        paragraphs: Vec<Paragraph<'a>>,
    },
    Javascript {
        contents: &'a str,
    },
    List {
        style: ListStyle,
        depth: usize,
        items: Vec<Paragraph<'a>>,
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
        paragraphs: Vec<Paragraph<'a>>,
    },
    Words {
        centered: bool,
        words: Vec<Word<'a>>,
    },
}

impl<'a> Paragraph<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>) -> Result<Self> {
        trace!("Converting pair into Paragraph...");

        // Handle outer wrapping
        let pair = match pair.as_rule() {
            Rule::paragraph => get_first_pair!(pair),
            Rule::paragraph_inner => pair,
            Rule::paragraphs_internal => {
                // This indicates a bug in the grammar
                panic!("The rule 'paragraphs_internal' returns multiple Paragraph instances")
            }
            _ => {
                return Err(Error::Msg(format!(
                    "Invalid rule for paragraph: {:?}",
                    pair.as_rule()
                )))
            }
        };

        match pair.as_rule() {
            Rule::paragraph_inner => Self::from_rule_inner(pair),
            Rule::just_newlines => Ok(Paragraph::Newlines {
                count: pair.as_str().len(),
            }),
            _ => panic!("Invalid rule for paragraph: {:?}", pair.as_rule()),
        }
    }

    fn from_rule_inner(pair: Pair<'a, Rule>) -> Result<Self> {
        debug_assert_eq!(pair.as_rule(), Rule::paragraph_inner);
        let pair = get_first_pair!(pair);

        let paragraph_inner = match pair.as_rule() {
            Rule::align => align::parse(pair)?,
            Rule::code => code::parse(pair)?,
            Rule::collapsible => collapsible::parse(pair)?,
            Rule::clear_float => clear_float::parse(pair),
            Rule::div => div::parse(pair)?,
            Rule::bullet_list | Rule::numbered_list => list::parse(pair)?,
            Rule::horizontal_line => Paragraph::HorizontalLine,
            Rule::html => Paragraph::Html {
                contents: extract!(HTML_BLOCK, pair),
            },
            Rule::iframe => iframe::parse(pair),
            Rule::javascript => Paragraph::Javascript {
                contents: extract!(JAVASCRIPT_BLOCK, pair),
            },
            Rule::quote_block => quote::parse(pair)?,
            Rule::words => words::parse(pair)?,

            _ => {
                return Err(Error::Msg(format!(
                    "Invalid rule for paragraph: {:?}",
                    pair.as_rule()
                )))
            }
        };

        Ok(paragraph_inner)
    }
}

impl<'a> AsRef<Paragraph<'a>> for Paragraph<'a> {
    #[inline]
    fn as_ref(&self) -> &Paragraph<'a> {
        self
    }
}

pub fn convert_internal_paragraphs(pair: Pair<Rule>) -> Result<Vec<Paragraph>> {
    let mut paragraphs = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::paragraph | Rule::paragraph_inner => {
                let paragraph = Paragraph::from_pair(pair)?;
                paragraphs.push(paragraph);
            }
            _ => panic!("Invalid rule for internal-paragraphs: {:?}", pair.as_rule()),
        }
    }

    Ok(paragraphs)
}

#[test]
fn test_regexes() {
    let _ = &*HTML_BLOCK;
    let _ = &*JAVASCRIPT_BLOCK;
}
