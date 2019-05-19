/*
 * parse/tree/word/module.rs
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

use std::collections::HashMap;
use super::prelude::*;

lazy_static! {
    static ref MODULE: Regex = {
        RegexBuilder::new(r"\[\[\s*module\s+[^\]]*\]\](?:\n(?P<contents>.*\n)\[\[\s*module\s*\]\])?")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
}

pub fn parse(pair: Pair<Rule>) -> Word {
    let mut name = "";
    let mut arguments = HashMap::new();

    let contents = MODULE
        .captures(pair.as_str())
        .expect("Regular expression MODULE didn't match")
        .name("contents")
        .map(|capture| capture.as_str());

    // Parse arguments
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::ident => name = pair.as_str(),
            Rule::module_arg => {
                let key = get_nth_pair!(pair, 0).as_str();
                let value = {
                    let pair = get_nth_pair!(pair, 1);
                    interp_str(pair.as_str()).expect("Invalid string value")
                };

                arguments.insert(key, value);
            }
            _ => panic!("Invalid rule for module: {:?}", pair.as_rule()),
        }
    }

    debug_assert_ne!(name, "", "Module name never set");

    Word::Module {
        name,
        arguments,
        contents,
    }
}

#[test]
fn test_regexes() {
    let _ = &*MODULE;
}
