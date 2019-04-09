/*
 * parse/mod.rs
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

mod filter;
mod string;
mod tree;

#[cfg(test)]
mod test;

pub use self::tree::{Line, SyntaxTree, Word};

use crate::Result;
use pest::Parser;
use pest::error::Error as PestError;

#[derive(Debug, Clone, Parser)]
#[grammar = "parse/wikidot.pest"]
pub struct WikidotParser;

pub type ParseError = PestError<Rule>;

pub fn parse<'a>(text: &'a str) -> Result<SyntaxTree<'a>> {
    let page = {
        // Should return exactly [ Rule::page ]
        //
        // The .next().unwrap().into_inner() pattern is used in cases
        // where a Pairs object wraps a single Pair object. For instance
        // here the Page rule only has one possibility, and the outer layer
        // is not useful for parsing, so we discard it.

        let mut page = WikidotParser::parse(Rule::page, text)?;
        page.next().unwrap().into_inner()
    };

    let tree = SyntaxTree::from_line_pairs(page);
    Ok(tree)
}
