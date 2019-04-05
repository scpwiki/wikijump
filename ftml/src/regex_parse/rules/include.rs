/*
 * parse/rules/include.rs
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

//! Processing rule for includes. Takes any other pages and copies their
//! contents into the current article for later styling and handling.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref INCLUDE: Regex = {
        RegexBuilder::new(r"\[\[include\s+(?P<page>[a-zA-Z0-9\s\-:]+?)(?P<args>\s+.*?)?\]\]$")
            .multi_line(true)
            .dot_matches_new_line(true)
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_include(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = INCLUDE.captures(state.text()) {
        let page = capture["page"].to_string();
        let args = capture.name("args").map(|mtch| mtch.as_str().to_string());
        let token = Token::Include { page, args };
        state.push_token(token, &*INCLUDE);
    }

    Ok(())
}

#[test]
fn test_include() {
    let mut state = ParseState::new("[[include component:special-embed-thing]]".into());
    rule_include(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("[[include\ncomponent:image-block\nname=\"file.png\"\ncaption=\"object\"]]".into());
    rule_include(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);
}
