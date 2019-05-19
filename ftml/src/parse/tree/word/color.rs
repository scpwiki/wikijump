/*
 * parse/tree/word/color.rs
 *
 * ftml - Convert Wikidot code to HTML
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

use super::prelude::*;

pub fn parse(pair: Pair<Rule>) -> Result<Word> {
    let mut color = "";
    let mut words = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::ident => color = pair.as_str(),
            Rule::word => {
                let word = Word::from_pair(pair)?;
                words.push(word);
            }
            _ => panic!("Invalid rule for color: {:?}", pair.as_rule()),
        }
    }

    Ok(Word::Color { color, words })
}
