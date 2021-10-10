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

mod flags;
mod mode;

use enumflags2::BitFlags;

pub use self::flags::ParserRender;
pub use self::mode::WikitextMode;

/// Settings to tweak behavior in the ftml parser and renderer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WikitextSettings {
    pub mode: WikitextMode,
    pub flags: BitFlags<ParserRender>,
}

impl WikitextSettings {
    pub fn set_mode(&mut self, mode: WikitextMode) {
        self.mode = mode;
        self.flags = mode.flags();
    }
}

impl Default for WikitextSettings {
    fn default() -> Self {
        todo!()
    }
}
