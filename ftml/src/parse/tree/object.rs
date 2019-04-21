/*
 * parse/tree/object.rs
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

use crate::Result;
use super::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree<'a> {
    lines: Vec<Line<'a>>,
}

impl<'a> SyntaxTree<'a> {
    pub fn from_line_pairs(pairs: Pairs<'a, Rule>) -> Result<Self> {
        trace!("Converting pairs into a SyntaxTree...");

        let lines_res: Result<Vec<_>> = pairs
            .into_iter()
            .filter(|pair| pair.as_rule() == Rule::line)
            .map(|pair| Line::from_pair(pair))
            .collect();

        lines_res.map(|lines| SyntaxTree { lines })
    }

    #[inline]
    pub fn lines(&self) -> &[Line] {
        self.lines.as_slice()
    }
}
