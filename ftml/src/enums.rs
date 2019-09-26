/*
 * enums.rs
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

use crate::StdResult;
use std::convert::TryFrom;
use std::fmt::{self, Display};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Alignment {
    Left,
    Right,
    Center,
    Justify,
}

impl Alignment {
    pub fn style(self) -> &'static str {
        match self {
            Alignment::Left => "left",
            Alignment::Right => "right",
            Alignment::Center => "center",
            Alignment::Justify => "justify",
        }
    }
}

impl<'a> TryFrom<&'a str> for Alignment {
    type Error = ();

    fn try_from(value: &'a str) -> StdResult<Self, Self::Error> {
        match value {
            "<" => Ok(Alignment::Left),
            ">" => Ok(Alignment::Right),
            "=" => Ok(Alignment::Center),
            "==" => Ok(Alignment::Justify),
            _ => Err(()),
        }
    }
}

impl Display for Alignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.style())
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AnchorTarget {
    NewTab,
    Parent,
    Top,
    Same,
}

impl AnchorTarget {
    pub fn style(self) -> &'static str {
        match self {
            AnchorTarget::NewTab => "_blank",
            AnchorTarget::Parent => "_parent",
            AnchorTarget::Top => "_top",
            AnchorTarget::Same => "_self",
        }
    }
}

impl<'a> TryFrom<&'a str> for AnchorTarget {
    type Error = ();

    fn try_from(value: &'a str) -> StdResult<Self, Self::Error> {
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

impl Display for AnchorTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.style())
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

impl TryFrom<usize> for HeadingLevel {
    type Error = ();

    fn try_from(value: usize) -> StdResult<Self, Self::Error> {
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

    fn try_from(value: u8) -> StdResult<Self, Self::Error> {
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

impl Display for HeadingLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let level = *self;
        let value: u8 = level.into();

        write!(f, "h{}", value)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LinkText<'a> {
    Text(&'a str),
    Url,
    Article,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ListStyle {
    Bullet,
    Numbered,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HtmlMetaType {
    Name,
    HttpEquiv,
    Property,
}

impl HtmlMetaType {
    pub fn tag_name(self) -> &'static str {
        use self::HtmlMetaType::*;

        match self {
            Name => "name",
            HttpEquiv => "http-equiv",
            Property => "property",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum InfoField {
    Title,
    AltTitle,
    Header,
    SubHeader,
}
