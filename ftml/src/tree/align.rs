/*
 * tree/align.rs
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

use regex::Regex;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Alignment {
    Left,
    Right,
    Center,
    Justify,
}

impl Alignment {
    pub fn name(self) -> &'static str {
        match self {
            Alignment::Left => "left",
            Alignment::Right => "right",
            Alignment::Center => "center",
            Alignment::Justify => "justify",
        }
    }
}

impl TryFrom<&'_ str> for Alignment {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<" => Ok(Alignment::Left),
            ">" => Ok(Alignment::Right),
            "=" => Ok(Alignment::Center),
            "==" => Ok(Alignment::Justify),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ImageAlignment {
    pub align: Alignment,
    pub float: bool,
}

impl ImageAlignment {
    pub fn parse(name: &str) -> Option<Self> {
        lazy_static! {
            static ref IMAGE_ALIGNMENT_REGEX: Regex = Regex::new(r"^f?([<=>])").unwrap();
        }

        IMAGE_ALIGNMENT_REGEX
            .find(name)
            .map(|mtch| ImageAlignment::try_from(mtch.as_str()).ok())
            .flatten()
    }

    pub fn class(self) -> &'static str {
        match (self.align, self.float) {
            (Alignment::Left, false) => "alignleft",
            (Alignment::Center, false) => "aligncenter",
            (Alignment::Right, false) => "alignright",
            (Alignment::Justify, false) => "alignjustify",
            (Alignment::Left, true) => "floatleft",
            (Alignment::Center, true) => "floatcenter",
            (Alignment::Right, true) => "floatright",
            (Alignment::Justify, true) => "floatjustify",
        }
    }
}

impl TryFrom<&'_ str> for ImageAlignment {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (align, float) = match value {
            "=" => (Alignment::Center, false),
            "<" => (Alignment::Left, false),
            ">" => (Alignment::Right, false),
            "f<" => (Alignment::Left, true),
            "f>" => (Alignment::Right, true),
            _ => return Err(()),
        };

        Ok(ImageAlignment { align, float })
    }
}
