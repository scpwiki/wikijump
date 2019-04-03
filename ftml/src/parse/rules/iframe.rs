/*
 * parse/rules/iframe.rs
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

//! Processing rule for iframes.
//! These are multi-line components using [[iframe]] to embed other websites.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref IFRAME: Regex = {
        RegexBuilder::new(r"^\[\[iframe\s+(?P<url>[^ ]+)(\s+.*?)?\]\]")
            .dot_matches_new_line(true)
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_iframe(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = IFRAME.captures(state.text()) {
        let url = capture["url"].to_string();
        let args = capture.name("args").map(|mtch| mtch.as_str().to_string());
        let token = Token::Iframe { url, args };
        state.push_token(token, &*IFRAME);
    }

    Ok(())
}

#[test]
fn test_iframe() {
    let mut state = ParseState::new("[[iframe]]\n".into());
    rule_iframe(&mut state).unwrap();
    assert_eq!(state.text(), "[[iframe]]\n");
    assert_eq!(state.tokens().len(), 0);

    let mut state = ParseState::new("[[iframe https://example.com/]]\n".into());
    rule_iframe(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0\n");
    assert_eq!(state.tokens().len(), 1);
}
