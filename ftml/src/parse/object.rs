/*
 * parse/object.rs
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

use super::SyntaxTree;
use crate::Result;
use pest::error::Error as PestError;
use pest::Parser;

#[derive(Debug, Copy, Clone, Parser)]
#[grammar = "parse/wikidot.pest"]
pub struct WikidotParser;

pub type ParseError = PestError<Rule>;

pub fn parse<'a>(text: &'a str) -> Result<SyntaxTree<'a>> {
    let page = {
        // Should return exactly [ Rule::page ]
        let mut pairs = WikidotParser::parse(Rule::page, text)?;
        get_inner_pairs!(pairs)
    };

    let tree = SyntaxTree::from_line_pairs(page)?;
    Ok(tree)
}
