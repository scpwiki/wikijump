/*
 * parse/tree/word/tab.rs
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

use super::prelude::*;
use super::super::misc::Tab;

pub fn parse(pair: Pair<Rule>) -> Result<Word> {
    let mut tabs = Vec::new();

    // Iterate over tabs
    for pair in pair.into_inner() {
        let mut pairs = pair.into_inner();
        let name = {
            let pair = pairs.next().expect("Tab pairs iterator was empty");
            pair.as_str()
        };
        let contents = {
            let pair = pairs
                .next()
                .expect("Tab pairs iterator had only one element");

            convert_internal_lines(pair)?
        };

        tabs.push(Tab { name, contents });
    }

    Ok(Word::TabList { tabs })
}
