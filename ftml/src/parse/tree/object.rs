/*
 * parse/tree/object.rs
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

use super::prelude::*;
use crate::Result;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree<'a> {
    #[serde(borrow)]
    paragraphs: Vec<Paragraph<'a>>,
}

impl<'a> SyntaxTree<'a> {
    pub fn from_line_pairs(pairs: Pairs<'a, Rule>) -> Result<Self> {
        trace!("Converting pairs into a SyntaxTree...");

        let result: Result<Vec<_>> = pairs
            .filter(|pair| pair.as_rule() == Rule::paragraph)
            .map(Paragraph::from_pair)
            .collect();

        result.map(|paragraphs| SyntaxTree { paragraphs })
    }

    pub fn from_paragraphs<I: Into<Vec<Paragraph<'a>>>>(paragraphs: I) -> Self {
        let paragraphs = paragraphs.into();

        SyntaxTree { paragraphs }
    }

    #[inline]
    pub fn paragraphs(&self) -> &[Paragraph] {
        self.paragraphs.as_slice()
    }
}
