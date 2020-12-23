/*
 * enums.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2020 Ammon Smith
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

// TODO use enums
#![allow(dead_code)]

use std::borrow::Cow;
use std::convert::TryFrom;
use strum_macros::IntoStaticStr;

#[derive(
    Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum AnchorTarget {
    /// Open the link in a new tab.
    /// HTML attribute is `_blank`.
    NewTab,

    /// Open the link in the parent frame.
    /// HTML attribute is `_parent`.
    Parent,

    /// Open the link in the top-most frame.
    /// HTML attribute is `_top`.
    Top,

    /// Open the link in the current frame.
    /// HTML attribute is `_self`.
    Same,
}

impl AnchorTarget {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}

impl<'a> TryFrom<&'a str> for AnchorTarget {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        const ANCHOR_TARGET_VALUES: [(&str, &str, AnchorTarget); 4] = [
            ("blank", "_blank", AnchorTarget::NewTab),
            ("parent", "_parent", AnchorTarget::Parent),
            ("top", "_top", AnchorTarget::Top),
            ("self", "_self", AnchorTarget::Same),
        ];

        for (value1, value2, target) in &ANCHOR_TARGET_VALUES {
            if value.eq_ignore_ascii_case(value1) || value.eq_ignore_ascii_case(value2) {
                return Ok(*target);
            }
        }

        Err(())
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HeadingLevel {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
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

impl Into<u8> for HeadingLevel {
    fn into(self) -> u8 {
        match self {
            HeadingLevel::One => 1,
            HeadingLevel::Two => 2,
            HeadingLevel::Three => 3,
            HeadingLevel::Four => 4,
            HeadingLevel::Five => 5,
            HeadingLevel::Six => 6,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LinkLabel<'a> {
    /// Custom text link label.
    ///
    /// Can be set to any arbitrary value of the input text's choosing.
    Text(Cow<'a, str>),

    /// URL-mirroring link label.
    ///
    /// The label for this link is the same as the URL it targets.
    Url,

    /// Article title-based link label.
    ///
    /// The label for this link is whatever the page's title is.
    Page,
}

#[derive(
    Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum ListStyle {
    Bullet,
    Numbered,
}

impl ListStyle {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}

#[derive(
    Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum InfoField {
    Title,
    Header,
    SubHeader,
}

impl InfoField {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}
