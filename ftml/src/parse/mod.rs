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
mod tree;

pub use self::tree::{Paragraph, SyntaxTree, Word};

use crate::{Error, Result};
use pest::Parser;

#[derive(Debug, Clone, Parser)]
#[grammar = "wikidot.pest"]
pub struct WikidotParser;

pub fn parse(_text: &str) -> Result<SyntaxTree> {
    Err(Error::StaticMsg("Not implemented yet"))
}

#[test]
fn test_parser() {
    const STRINGS: [&str; 4] = [
        "@@ test raw str @@ @@ second raw @@",
        "__**test** string {{ here }}__ ^^up!^^",
        "**[[date 0]]**",
        "[[span class=\"test\"]]//hello// world![[footnote]]actually country[[/footnote]][[/span]]",
    ];

    for string in &STRINGS[..] {
        let parse_result = WikidotParser::parse(Rule::page, string);
        println!("> \"{}\"\n{:#?}", string, &parse_result);
    }
}
