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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Center,
    Justify,
}

// Convert to TryFrom
impl Alignment {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "<" => Some(Alignment::Left),
            ">" => Some(Alignment::Right),
            "=" => Some(Alignment::Center),
            "==" => Some(Alignment::Justify),
            _ => None,
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

// Convert to Into / TryFrom
impl HeadingLevel {
    pub fn from_usize(n: usize) -> Option<Self> {
        match n {
            1 => Some(HeadingLevel::One),
            2 => Some(HeadingLevel::Two),
            3 => Some(HeadingLevel::Three),
            4 => Some(HeadingLevel::Four),
            5 => Some(HeadingLevel::Five),
            6 => Some(HeadingLevel::Six),
            _ => None,
        }
    }

    pub fn to_usize(self) -> usize {
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
