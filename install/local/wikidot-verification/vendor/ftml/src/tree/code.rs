/*
 * code.rs
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

//! Structure to represent a code block.

use super::clone::{option_string_to_owned, string_to_owned};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeBlock<'t> {
    pub contents: Cow<'t, str>,
    pub language: Option<Cow<'t, str>>,
    pub name: Option<Cow<'t, str>>,
}

impl CodeBlock<'_> {
    pub fn to_owned(&self) -> CodeBlock<'static> {
        CodeBlock {
            contents: string_to_owned(&self.contents),
            language: option_string_to_owned(&self.language),
            name: option_string_to_owned(&self.name),
        }
    }
}
