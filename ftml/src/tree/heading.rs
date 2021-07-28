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
