/*
 * parse/rules/prefilter.rs
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

//! Pre-filtering processing rule. Contains several minor changes, such as:
//! * Converting to UNIX line endings
//! * Trim excess whitespace
//! * Converting tabs to 4-spaces
//! * Adding newlines to the top and bottom of the text
//! * Compressing 3+ newlines into 2 newlines

use crate::{ParseState, Result};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref EXCESS_WHITESPACE: Regex = {
        RegexBuilder::new(r"^\s+$")
            .multi_line(true)
            .build()
            .unwrap()
    };

    static ref MULTIPLE_NEWLINES: Regex = {
        RegexBuilder::new(r"(\n\s*){3,}")
            .multi_line(true)
            .build()
            .unwrap()
    };
}

pub fn rule_prefilter(state: &mut ParseState) -> Result<()> {
    // DOS line endings
    state.ireplace_all("\r\n", "\n");

    // Old Mac line endings
    state.ireplace_all("\r", "\n");

    // Trim excess whitespace
    state.ireplace_all_regex(&*EXCESS_WHITESPACE, "");

    // Convert tabs
    state.ireplace_all("\t", "    ");

    // Add newlines to the top and the bottom
    state.insert(0, '\n');
    state.push('\n');

    // Compress 3+ newlines into 2 newlines
    state.ireplace_all_regex(&*MULTIPLE_NEWLINES, "\n\n");

    Ok(())
}

#[test]
fn test_whitespace() {
    let mut state = ParseState::new("Apple\rBanana\r\nCherry\tDurian".into());
    rule_prefilter(&mut state).unwrap();
    assert_eq!(state.text(), "\nApple\nBanana\nCherry    Durian\n");

    let mut state = ParseState::new("Apple\n\n\n\nBanana\n\r\rCherry\n\nDurian\nPineapple".into());
    rule_prefilter(&mut state).unwrap();
    assert_eq!(state.text(), "\nApple\n\nBanana\n\nCherry\n\nDurian\nPineapple\n");
}
