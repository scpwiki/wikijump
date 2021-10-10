/*
 * settings/flags.rs
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

use super::WikitextMode;

bitflags! {
    pub struct WikitextFlags: u32 {
        const NONE            = 0b0000_0000_0000_0000;

        // Whether certain syntactical constructs are allowed
        const ALLOW_INCLUDE   = 0b0000_0000_0000_0001;
        const ALLOW_MODULE    = 0b0000_0000_0000_0010;
        const ALLOW_TOC       = 0b0000_0000_0000_0100;
        const ALLOW_BUTTON    = 0b0000_0000_0000_1000;

        // Whether real IDs should be used (true), or randomly generated ones (false)
        const HEADING_ID      = 0b0000_0000_0001_0000;
        const FOOTNOTE_ID     = 0b0000_0000_0010_0000;
        const BIBLIOGRAPHY_ID = 0b0000_0000_0100_0000;
        const MATH_ID         = 0b0000_0000_1000_0000;

        // Whether local paths are permitted.
        const ALLOW_LOCAL     = 0b0000_0001_0000_0000;
    }
}

impl Default for WikitextFlags {
    #[inline]
    fn default() -> Self {
        WikitextMode::default().flags()
    }
}
