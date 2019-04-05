/*
 * parse/rules/heading.rs
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

//! Processing rule for headings, basically what HTML calls `<h1>` through `<h6>`.
//!

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref HEADING: Regex = {
        RegexBuilder::new(r"^(?P<level>\+{1,6})\s+(?P<contents>.*)$")
            .multi_line(true)
            .build()
            .unwrap()
    };
}

pub fn rule_heading(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = HEADING.captures(state.text()) {
        let level = capture["level"].len() as u8;
        let contents = capture["contents"].to_string();
        let token = Token::Heading { contents, level };
        state.push_token(token, &*HEADING);
    }

    Ok(())
}

#[test]
fn test_heading() {
    let mut state = ParseState::new("++ Apple\nBanana".into());
    rule_heading(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0\nBanana");
    assert_eq!(state.tokens().len(), 1);

    match state.token(0) {
        Some(Token::Heading { contents, level }) => {
            assert_eq!(contents, "Apple");
            assert_eq!(*level, 2);
        }
        Some(token) => panic!("Token not heading, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    let mut state = ParseState::new("++++++ Cherry\n+ Durian\n++++++++++ Pineapple".into());
    rule_heading(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0\n\01\0\n++++++++++ Pineapple");
    assert_eq!(state.tokens().len(), 2);

    match state.token(0) {
        Some(Token::Heading { contents, level }) => {
            assert_eq!(contents, "Cherry");
            assert_eq!(*level, 6);
        }
        Some(token) => panic!("Token not heading, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    match state.token(1) {
        Some(Token::Heading { contents, level }) => {
            assert_eq!(contents, "Durian");
            assert_eq!(*level, 1);
        }
        Some(token) => panic!("Token not heading, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }
}
