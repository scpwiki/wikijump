/*
 * settings/mod.rs
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

mod mode;

pub use self::mode::WikitextMode;

/// Settings to tweak behavior in the ftml parser and renderer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct WikitextSettings {
    /// What mode we're running in.
    pub mode: WikitextMode,

    /// Whether page-contextual syntax is permitted.
    ///
    /// This currently refers to:
    /// * Include
    /// * Module
    /// * Table of Contents
    /// * Button
    pub enable_page_syntax: bool,

    /// Whether IDs should be randomly-generated, or true IDs should be used.
    ///
    /// In the latter case, IDs can be used for navigation, for instance
    /// the table of contents, but setting this to `true` is needed in any
    /// context where more than one instance of rendered wikitext could be emitted.
    pub use_random_ids: bool,

    /// Whether local paths are permitted.
    ///
    /// This applies to:
    /// * Files
    /// * Images
    pub allow_local_paths: bool,
}

impl WikitextSettings {
    pub fn set_mode(&mut self, mode: WikitextMode) {
        self.mode = mode;

        match mode {
            WikitextMode::Page | WikitextMode::Draft => {
                self.enable_page_syntax = true;
                self.use_random_ids = false;
                self.allow_local_paths = true;
            },
            WikitextMode::ForumPost | WikitextMode::DirectMessage => {
                self.enable_page_syntax = false;
                self.use_random_ids = true;
                self.allow_local_paths = false;
            },
            WikitextMode::List => {
                self.enable_page_syntax = true;
                self.use_random_ids = true;
            },
        }
    }
}

impl Default for WikitextSettings {
    fn default() -> Self {
        WikitextSettings {
            mode: WikitextMode::default(),
            enable_page_syntax: true,
            use_random_ids: false,
            allow_local_paths: true,
        }
    }
}
