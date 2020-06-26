/*
 * parse/data.rs
 *
 * ftml - Library to parse Wikidot code
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

use crate::enums::Alignment;
use regex::Regex;

lazy_static! {
    static ref IMAGE_ALIGNMENT_REGEX: Regex = Regex::new(r"(f?[<>])|=").unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ImageAlignment {
    pub align: Alignment,
    pub float: bool,
}

impl ImageAlignment {
    pub fn parse(text: &str) -> Option<Self> {
        let (align, float) = match text {
            "=" => (Alignment::Center, false),
            "<" => (Alignment::Left, false),
            ">" => (Alignment::Right, false),
            "f<" => (Alignment::Left, true),
            "f>" => (Alignment::Right, true),
            _ => return None,
        };

        Some(ImageAlignment { align, float })
    }
}
