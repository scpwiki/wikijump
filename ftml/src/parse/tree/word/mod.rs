/*
 * parse/tree/word/mod.rs
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

macro_rules! make_words {
    ($pair:expr) => {{
        // Turns a Vec<Result<_>> into Result<Vec<_>>
        let word_res: Result<Vec<_>> = $pair.into_inner().map(Word::from_pair).collect();

        word_res?
    }};
}

// Em dash character: '—'
const EM_DASH: &str = "\u{2014}";

mod anchor;
mod color;
mod image;
mod link;
mod module;
mod span;
mod tab;

mod prelude {
    pub use super::super::prelude::*;
    pub use crate::enums::{AnchorTarget, InfoField, LinkText};
    pub use crate::{Error, Result};
    pub use std::borrow::Cow;
    pub use std::convert::TryFrom;

    pub fn get_link_target(pair: Pair<Rule>) -> Option<AnchorTarget> {
        debug_assert_eq!(pair.as_rule(), Rule::link_newtab);

        match pair.as_str() {
            "*" => Some(AnchorTarget::NewTab),
            "" => None,
            value => panic!("Invalid value for Rule::link_newtab: {:?}", value),
        }
    }
}

use self::prelude::*;
use crate::enums::Alignment;
use std::collections::HashMap;

lazy_static! {
    static ref ANCHOR: Regex = {
        RegexBuilder::new(r"\[\[#\s*([a-z0-9\-+_.%]+)\s*\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };
    static ref CSS: Regex = {
        RegexBuilder::new(
            r#"(?x)
            \[\[\s*(?:css|style)\s*\]\]\n
            (?P<style>.*)\n
            \[\[/\s*(?:css|style)\s*\]\]"#,
        )
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
    };
    static ref DATE: Regex = {
        RegexBuilder::new(
            r#"(?x)
            \[\[
                \s*date\s+
                (?P<timestamp>-?[0-9]+)
                (?:\s+format\s*=\s*"
                    (?P<format>.*)
                ")?
                \s*
            \]\]"#,
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
        RegexBuilder::new(
            r"(?x)
            \[\[\s*form\s*\]\]\n
                (?P<contents>(?:.*\n)?)
            \[\[/\s*form\s*\]\]",
        )
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
    };
    static ref RAW: Regex = {
        RegexBuilder::new(r"^@[@<](?P<contents>.*)[>@]@$")
            .build()
            .unwrap()
    };
    static ref STRIKETHROUGH: Regex = Regex::new(r"--(?P<contents>.+)--").unwrap();
    static ref USER: Regex = {
        RegexBuilder::new(r"\[\[\s*(?P<picture>\*)?\s*user\s+(?P<username>[^ ]+)\s*\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Word<'a> {
    Anchor {
        href: Option<Cow<'a, str>>,
        name: Option<Cow<'a, str>>,
        id: Option<Cow<'a, str>>,
        class: Option<Cow<'a, str>>,
        style: Option<Cow<'a, str>>,
        target: Option<AnchorTarget>,
        words: Vec<Word<'a>>,
    },
    Bold {
        words: Vec<Word<'a>>,
    },
    Color {
        color: &'a str,
        words: Vec<Word<'a>>,
    },
    Css {
        style: &'a str,
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
        text: Option<&'a str>,
        target: Option<AnchorTarget>,
    },
    Footnote {
        paragraphs: Vec<Paragraph<'a>>,
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
        alt: Option<Cow<'a, str>>,
        title: Option<Cow<'a, str>>,
        width: Option<Cow<'a, str>>,
        height: Option<Cow<'a, str>>,
        style: Option<Cow<'a, str>>,
        class: Option<Cow<'a, str>>,
        size: Option<Cow<'a, str>>,
    },
    Info {
        field: InfoField,
    },
    Italics {
        words: Vec<Word<'a>>,
    },
    Link {
        href: &'a str,
        target: Option<AnchorTarget>,
        text: LinkText<'a>,
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
        paragraphs: Vec<Paragraph<'a>>,
    },
    Raw {
        contents: &'a str,
    },
    Size {
        size: &'a str,
        paragraphs: Vec<Paragraph<'a>>,
    },
    Span {
        id: Option<Cow<'a, str>>,
        class: Option<Cow<'a, str>>,
        style: Option<Cow<'a, str>>,
        paragraphs: Vec<Paragraph<'a>>,
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
        let word = match pair.as_rule() {
            Rule::text => Word::Text {
                contents: pair.as_str(),
            },
            Rule::raw => Word::Raw {
                contents: extract!(RAW, pair),
            },
            Rule::email => Word::Email {
                address: pair.as_str(),
                text: None,
            },
            Rule::em_dash => Word::Text { contents: EM_DASH },
            Rule::color => color::parse(pair)?,
            Rule::italics => Word::Italics {
                words: make_words!(pair),
            },
            Rule::strikethrough => {
                // This rule is essentially a ".+" and then parse out the words.

                use pest::Parser;

                let contents = extract!(STRIKETHROUGH, pair);
                let mut pairs = WikidotParser::parse(Rule::strikethrough_words, contents)?;
                pairs = get_inner_pairs!(pairs);

                let mut words = Vec::new();
                for pair in pairs {
                    if pair.as_rule() == Rule::word {
                        let word = Word::from_pair(pair)?;
                        words.push(word);
                    }
                }

                Word::Strikethrough { words }
            }
            Rule::bold => Word::Bold {
                words: make_words!(pair),
            },
            Rule::underline => Word::Underline {
                words: make_words!(pair),
            },
            Rule::subscript => Word::Subscript {
                words: make_words!(pair),
            },
            Rule::superscript => Word::Superscript {
                words: make_words!(pair),
            },
            Rule::monospace => Word::Monospace {
                words: make_words!(pair),
            },
            Rule::anchor => {
                // The [# LINK] anchor, which only has a name.
                // Compare to the general-purpose [[a]] anchor
                // in the rule below this one.
                Word::Anchor {
                    href: None,
                    name: Some(Cow::Borrowed(extract!(ANCHOR, pair))),
                    id: None,
                    class: None,
                    style: None,
                    target: None,
                    words: Vec::new(),
                }
            }
            Rule::anchor_tag => anchor::parse(pair)?,
            Rule::css => Word::Css {
                style: extract!(CSS, pair),
            },
            Rule::date => {
                let capture = DATE
                    .captures(pair.as_str())
                    .expect("Regular expression DATE didn't match");

                Word::Date {
                    timestamp: capture["timestamp"]
                        .parse()
                        .expect("Unable to parse timestamp integer"),
                    format: capture.name("format").map(|mtch| mtch.as_str()),
                }
            }
            Rule::equation_ref => Word::EquationReference {
                name: extract!(EQUATION_REF, pair),
            },
            Rule::file_ref => link::parse_file(pair),
            Rule::footnote => Word::Footnote {
                paragraphs: convert_internal_paragraphs(get_first_pair!(pair))?,
            },
            Rule::footnote_block => Word::FootnoteBlock,
            Rule::form => Word::Form {
                contents: extract!(FORM, pair),
            },
            Rule::gallery => Word::Gallery,
            Rule::title => Word::Info {
                field: InfoField::Title,
            },
            Rule::alt_title => Word::Info {
                field: InfoField::AltTitle,
            },
            Rule::header => Word::Info {
                field: InfoField::Header,
            },
            Rule::subheader => Word::Info {
                field: InfoField::SubHeader,
            },
            Rule::module => module::parse(pair),
            Rule::note => {
                let mut paragraphs = Vec::new();

                for pair in pair.into_inner() {
                    let paragraph = Paragraph::from_pair(pair)?;
                    paragraphs.push(paragraph);
                }

                Word::Note { paragraphs }
            }
            Rule::image => image::parse(pair)?,
            Rule::link_bare => link::parse_bare(pair),
            Rule::link_page => link::parse_page(pair),
            Rule::link_url => link::parse_url(pair),
            Rule::size => {
                let mut pairs = pair.into_inner();
                let size = {
                    let pair = pairs.next().expect("Size pairs iterator was empty");
                    pair.as_str()
                };
                let paragraphs = {
                    let pair = pairs
                        .next()
                        .expect("Size pairs iterator had only one element");

                    convert_internal_paragraphs(pair)?
                };

                Word::Size { size, paragraphs }
            }
            Rule::span => span::parse(pair)?,
            Rule::tab_list => tab::parse(pair)?,
            Rule::user => {
                let capture = USER
                    .captures(pair.as_str())
                    .expect("Regular expression USER didn't match");

                Word::User {
                    username: capture!(capture, "username"),
                    show_picture: capture.name("picture").is_some(),
                }
            }
            _ => {
                return Err(Error::Msg(format!(
                    "Invalid rule for word: {:?}",
                    pair.as_rule()
                )))
            }
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

#[test]
fn test_regexes() {
    let _ = &*ANCHOR;
    let _ = &*DATE;
    let _ = &*EQUATION_REF;
    let _ = &*FILENAME;
    let _ = &*FORM;
    let _ = &*RAW;
    let _ = &*USER;
}
