/*
 * parse/rules/blockquote.rs
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

//! Processing rule for blockquotes.
//! These work by starting the line with some number of `>` characters followed by a space.
//!
//! Wikidot is a pain about it always requiring spaces at the end, so we should make it more
//! lax once the library is more widely used.

use crate::{ParseState, Result, Token};
use crate::parse::BlockQuoteLine;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref BLOCKQUOTE: Regex = {
        RegexBuilder::new(r"^(?:>+ .+\n)+")
            .multi_line(true)
            .build()
            .unwrap()
    };

    static ref BLOCKQUOTE_LINE: Regex = {
        RegexBuilder::new(r"^(?P<depth>>+) (?P<contents>.*)$")
            .multi_line(true)
            .build()
            .unwrap()
    };
}

pub fn rule_blockquote(state: &mut ParseState) -> Result<()> {
    while let Some(mtch) = BLOCKQUOTE.find(state.text()) {
        let mut lines = Vec::new();

        for capture in BLOCKQUOTE_LINE.captures_iter(mtch.as_str()) {
            let depth = capture["depth"].len();
            let contents = capture["contents"].to_string();
            let line = BlockQuoteLine { contents, depth };
            lines.push(line);
        }

        let token = Token::BlockQuote { lines };
        state.push_token(token, &*BLOCKQUOTE);
    }

    Ok(())
}

#[test]
fn test_blockquote() {
    let mut state = ParseState::new("[\n> apple\n> banana\n>> cherry\n>> durian\n> pineapple\n]".into());
    rule_blockquote(&mut state).unwrap();
    assert_eq!(state.text(), "[\n\00\0]");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new(">apple\n>\n>  banana\n> cherry\n>>durian\n".into());
    rule_blockquote(&mut state).unwrap();
    assert_eq!(state.text(), ">apple\n>\n\00\0>>durian\n");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("> apple\n\n> banana\n\n> cherry\n".into());
    rule_blockquote(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0\n\01\0\n\02\0");
    assert_eq!(state.tokens().len(), 3);

    let mut state = ParseState::new(">>> very deep quote block\n> look at quote\n> apple\n".into());
    rule_blockquote(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);

    match state.token(0) {
        Some(Token::BlockQuote { lines }) => {
            assert_eq!(lines.len(), 3);

            assert_eq!(lines[0].depth, 3);
            assert_eq!(lines[0].contents, "very deep quote block");
            assert_eq!(lines[1].depth, 1);
            assert_eq!(lines[1].contents, "look at quote");
            assert_eq!(lines[2].depth, 1);
            assert_eq!(lines[2].contents, "apple");
        }
        Some(token) => panic!("Token not blockquote, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }
}
