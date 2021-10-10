/*
 * settings/mode.rs
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

use super::WikitextFlags;

/// What mode parsing and rendering is done in.
///
/// Each variant has slightly different behavior associated
/// with them, beyond the typical flags for the rest of `WikitextSettings`.
///
/// The exact details of each are still being decided as this is implemented.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum WikitextMode {
    Page,
    ForumPost,
    DirectMessage,
    List,
}

impl WikitextMode {
    pub fn flags(self) -> WikitextFlags {
        match self {
            WikitextMode::Page => {
                WikitextFlags::ALLOW_INCLUDE
                    | WikitextFlags::ALLOW_MODULE
                    | WikitextFlags::ALLOW_TOC
                    | WikitextFlags::ALLOW_BUTTON
                    | WikitextFlags::HEADING_ID
                    | WikitextFlags::FOOTNOTE_ID
                    | WikitextFlags::BIBLIOGRAPHY_ID
                    | WikitextFlags::MATH_ID
                    | WikitextFlags::ALLOW_LOCAL
            }
            WikitextMode::ForumPost | WikitextMode::DirectMessage => WikitextFlags::NONE,
            WikitextMode::List => {
                WikitextFlags::ALLOW_INCLUDE
                    | WikitextFlags::ALLOW_MODULE
                    | WikitextFlags::ALLOW_TOC
                    | WikitextFlags::ALLOW_BUTTON
                    | WikitextFlags::ALLOW_LOCAL
            }
        }
    }
}

impl Default for WikitextMode {
    #[inline]
    fn default() -> Self {
        WikitextMode::Page
    }
}

impl From<WikitextMode> for WikitextFlags {
    #[inline]
    fn from(mode: WikitextMode) -> WikitextFlags {
        mode.flags()
    }
}
