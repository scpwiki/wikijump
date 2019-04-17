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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ListStyle {
    Bullet,
    Numbered,
}
