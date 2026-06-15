/*
 * tree/clear_float.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ClearFloat {
    Left,
    Right,
    Both,
}

impl ClearFloat {
    pub fn wd_html_style(self) -> &'static str {
        match self {
            ClearFloat::Left => "clear:left; height: 0px; font-size: 1px",
            ClearFloat::Right => "clear:right; height: 0px; font-size: 1px",
            // This is the only variant possible in Wikidot itself
            ClearFloat::Both => "clear:both; height: 0px; font-size: 1px",
        }
    }

    pub fn wj_html_class(self) -> &'static str {
        match self {
            ClearFloat::Left => "wj-clear-float-left",
            ClearFloat::Right => "wj-clear-float-right",
            ClearFloat::Both => "wj-clear-float-both",
        }
    }
}
