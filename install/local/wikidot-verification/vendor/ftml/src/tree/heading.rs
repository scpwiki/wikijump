/*
 * tree/heading.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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

use super::HtmlTag;
use crate::next_index::{NextIndex, TableOfContentsIndex};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Heading {
    /// The depth that this heading extends to.
    ///
    /// See [`HeadingLevel`].
    ///
    /// [`HeadingLevel`]: ./enum.HeadingLevel.html
    pub level: HeadingLevel,

    /// Whether this heading should get a table of contents entry or not.
    pub has_toc: bool,
}

impl Heading {
    pub fn html_tag(self, indexer: &mut dyn NextIndex<TableOfContentsIndex>) -> HtmlTag {
        let tag = self.level.html_tag();

        if self.has_toc
            && let Some(index) = indexer.next()
        {
            let id = format!("toc{index}");
            HtmlTag::with_id(tag, id)
        } else {
            HtmlTag::new(tag)
        }
    }
}

impl TryFrom<&'_ str> for Heading {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Headings take the form "\+{1,6}\*?" (regex)
        // The trailing "*" means that the TOC is *not* applied.
        // The heading depth is simply the ASCII length of "+" characters.
        //
        // This does *not* validate the regex, it assumes the string fits.

        let last_char = value.chars().next_back().ok_or(())?;
        let (has_toc, len) = match last_char {
            '+' => (true, value.len()),
            '*' => (false, value.len() - 1),
            _ => return Err(()),
        };

        let level = HeadingLevel::try_from(len)?;
        Ok(Heading { level, has_toc })
    }
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum HeadingLevel {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
}

impl HeadingLevel {
    #[inline]
    pub fn value(self) -> u8 {
        match self {
            HeadingLevel::One => 1,
            HeadingLevel::Two => 2,
            HeadingLevel::Three => 3,
            HeadingLevel::Four => 4,
            HeadingLevel::Five => 5,
            HeadingLevel::Six => 6,
        }
    }

    #[inline]
    pub fn prefix(self) -> &'static str {
        match self {
            HeadingLevel::One => "+",
            HeadingLevel::Two => "++",
            HeadingLevel::Three => "+++",
            HeadingLevel::Four => "++++",
            HeadingLevel::Five => "+++++",
            HeadingLevel::Six => "++++++",
        }
    }

    #[inline]
    pub fn prefix_with_space(self) -> &'static str {
        match self {
            HeadingLevel::One => "+ ",
            HeadingLevel::Two => "++ ",
            HeadingLevel::Three => "+++ ",
            HeadingLevel::Four => "++++ ",
            HeadingLevel::Five => "+++++ ",
            HeadingLevel::Six => "++++++ ",
        }
    }

    #[inline]
    pub fn html_tag(self) -> &'static str {
        match self {
            HeadingLevel::One => "h1",
            HeadingLevel::Two => "h2",
            HeadingLevel::Three => "h3",
            HeadingLevel::Four => "h4",
            HeadingLevel::Five => "h5",
            HeadingLevel::Six => "h6",
        }
    }
}

impl From<HeadingLevel> for HtmlTag {
    #[inline]
    fn from(level: HeadingLevel) -> HtmlTag {
        HtmlTag::new(level.html_tag())
    }
}

impl TryFrom<usize> for HeadingLevel {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(HeadingLevel::One),
            2 => Ok(HeadingLevel::Two),
            3 => Ok(HeadingLevel::Three),
            4 => Ok(HeadingLevel::Four),
            5 => Ok(HeadingLevel::Five),
            6 => Ok(HeadingLevel::Six),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for HeadingLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(HeadingLevel::One),
            2 => Ok(HeadingLevel::Two),
            3 => Ok(HeadingLevel::Three),
            4 => Ok(HeadingLevel::Four),
            5 => Ok(HeadingLevel::Five),
            6 => Ok(HeadingLevel::Six),
            _ => Err(()),
        }
    }
}

#[test]
fn heading() {
    macro_rules! test {
        ($input:expr, $level:expr, $has_toc:expr) => {{
            use std::convert::TryInto;

            let level = ($level as u8)
                .try_into()
                .expect("Heading level value was invalid");

            let heading =
                Heading::try_from($input).expect("Parsing heading token string failed");

            assert_eq!(heading.level, level, "Heading level doesn't match expected");
            assert_eq!(
                heading.has_toc, $has_toc,
                "Heading table of contents value doesn't match expected",
            );
        }};
    }

    test!("+", 1, true);
    test!("++", 2, true);
    test!("+++", 3, true);
    test!("++++", 4, true);
    test!("+++++", 5, true);
    test!("++++++", 6, true);

    test!("+*", 1, false);
    test!("++*", 2, false);
    test!("+++*", 3, false);
    test!("++++*", 4, false);
    test!("+++++*", 5, false);
    test!("++++++*", 6, false);
}

#[test]
fn true_ids() {
    use crate::next_index::Incrementer;

    macro_rules! test {
        ($indexer:expr, $level:expr, $expected_html_tag:expr $(,)?) => {{
            let level =
                HeadingLevel::try_from($level as u8).expect("Unable to get HeadingLevel");

            let heading = Heading {
                level,
                has_toc: true,
            };

            let actual_html_tag = heading.html_tag(&mut $indexer);
            assert_eq!(
                actual_html_tag, $expected_html_tag,
                "Actual HtmlTag didn't match expected",
            );
        }};
    }

    // Enabled incrementer
    // Simulates use_true_ids = true
    {
        let mut indexer = Incrementer::default();
        test!(
            indexer,
            1,
            HtmlTag::TagAndId {
                tag: "h1",
                id: str!("toc0")
            },
        );
        test!(
            indexer,
            3,
            HtmlTag::TagAndId {
                tag: "h3",
                id: str!("toc1")
            },
        );
        test!(
            indexer,
            3,
            HtmlTag::TagAndId {
                tag: "h3",
                id: str!("toc2")
            },
        );
        test!(
            indexer,
            1,
            HtmlTag::TagAndId {
                tag: "h1",
                id: str!("toc3")
            },
        );
        test!(
            indexer,
            5,
            HtmlTag::TagAndId {
                tag: "h5",
                id: str!("toc4")
            },
        );
    }

    // Disabled incrementer
    // Simulates use_true_ids = false
    {
        let mut indexer = Incrementer::disabled();
        test!(indexer, 1, HtmlTag::Tag("h1"));
        test!(indexer, 3, HtmlTag::Tag("h3"));
        test!(indexer, 3, HtmlTag::Tag("h3"));
        test!(indexer, 2, HtmlTag::Tag("h2"));
        test!(indexer, 4, HtmlTag::Tag("h4"));
    }
}
