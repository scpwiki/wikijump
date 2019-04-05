/*
 * parse/rules/span.rs
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

//! Processing rule for `[[span]]` blocks, which basically act
//! like directly inserted HTML tags of the same name.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref SPAN: Regex = {
        RegexBuilder::new(r"\[\[span(?P<args>\s+[^\]]*)?\]\](?P<contents>.*?)\[\[/span\]\]")
            .multi_line(true)
            .dot_matches_new_line(true)
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_span(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = SPAN.captures(state.text()) {
        let args = capture.name("args").map(|mtch| mtch.as_str().to_string());
        let contents = capture["contents"].to_string();
        let token = Token::Span { args, contents };
        state.push_token(token, &*SPAN);
    }

    Ok(())
}

#[test]
fn test_span() {
    let mut state = ParseState::new("durian [[span]]apple[[/span]] banana".into());
    rule_span(&mut state).unwrap();
    assert_eq!(state.text(), "durian \00\0 banana");
    assert_eq!(state.tokens().len(), 1);

    match state.token(0) {
        Some(Token::Span { args, contents }) => {
            assert_eq!(args.is_none(), true);
            assert_eq!(contents, "apple");
        }
        Some(token) => panic!("Token not span, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    let mut state = ParseState::new("[[span id=\"test\" class=\"fruit\"]]cherry[[/span]]".into());
    rule_span(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);

    match state.token(0) {
        Some(Token::Span { args, contents }) => {
            assert_eq!(args.is_some(), true);
            assert_eq!(args.as_ref().unwrap(), " id=\"test\" class=\"fruit\"");
            assert_eq!(contents, "cherry");
        }
        Some(token) => panic!("Token not span, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }
}
