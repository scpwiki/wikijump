/*
 * tree/heading.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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
    macro_rules! check {
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

    check!("+", 1, true);
    check!("++", 2, true);
    check!("+++", 3, true);
    check!("++++", 4, true);
    check!("+++++", 5, true);
    check!("++++++", 6, true);

    check!("+*", 1, false);
    check!("++*", 2, false);
    check!("+++*", 3, false);
    check!("++++*", 4, false);
    check!("+++++*", 5, false);
    check!("++++++*", 6, false);
}
