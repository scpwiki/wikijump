/*
 * enums.rs
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

use crate::StdResult;
use std::convert::TryFrom;
use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Center,
    Justify,
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
        let style = match *self {
            Alignment::Left => "left",
            Alignment::Right => "right",
            Alignment::Center => "center",
            Alignment::Justify => "justify",
        };

        write!(f, "{}", style)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum AnchorTarget {
    NewTab,
    Parent,
    Top,
    Same,
}

impl<'a> TryFrom<&'a str> for AnchorTarget {
    type Error = ();

    fn try_from(value: &'a str) -> StdResult<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "blank" | "_blank" => Ok(AnchorTarget::NewTab),
            "parent" | "_parent" => Ok(AnchorTarget::Parent),
            "top" | "_top" => Ok(AnchorTarget::Top),
            "self" | "_self" | "" => Ok(AnchorTarget::Same),
            _ => Err(()),
        }
    }
}

impl Display for AnchorTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let style = match *self {
            AnchorTarget::NewTab => "_blank",
            AnchorTarget::Parent => "_parent",
            AnchorTarget::Top => "_top",
            AnchorTarget::Same => "_self",
        };

        write!(f, "{}", style)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum HeadingLevel {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
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

impl Into<usize> for HeadingLevel {
    fn into(self) -> usize {
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
        let value: usize = level.into();

        write!(f, "h{}", value)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ListStyle {
    Bullet,
    Numbered,
}

impl Display for ListStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tag = match *self {
            ListStyle::Bullet => "ul",
            ListStyle::Numbered => "ol",
        };

        write!(f, "{}", tag)
    }
}
