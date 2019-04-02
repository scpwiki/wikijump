/*
 * parse/token.rs
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

use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum Token {
    CodeBlock {
        args: Option<String>,
        contents: String,
    },
    Form {
        contents: String,
    },
    Module {
        name: String,
        args: Option<String>,
        contents: Option<String>,
    },
    Raw {
        contents: String,
    },
}

#[must_use = "token ids should be inserted into the state string"]
#[derive(Debug, PartialEq, Eq)]
pub struct TokenId(usize);

impl TokenId {
    #[inline]
    pub fn new(id: usize) -> Self {
        TokenId(id)
    }

    #[inline]
    pub fn get(&self) -> usize {
        self.0
    }
}

impl Display for TokenId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\0{}\0", self.0)
    }
}
