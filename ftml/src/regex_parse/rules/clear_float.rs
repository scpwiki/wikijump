/*
 * parse/rules/clear_float.rs
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

//! Processing rule for `~~~~`, which clears floats from `div`s.
//! See https://www.wikidot.com/doc-wiki-syntax:block-formatting-elements

use crate::{ParseState, Result, Token};
use crate::parse::Direction;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref CLEAR_FLOAT: Regex = {
        RegexBuilder::new(r"^~{4,}(?P<direction><|>)?$")
            .multi_line(true)
            .build()
            .unwrap()
    };
}

pub fn rule_clear_float(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = CLEAR_FLOAT.captures(state.text()) {
        let direction = capture.name("direction").map(|s|
            match s.as_str() {
                "<" => Direction::Left,
                ">" => Direction::Right,
                _ => unreachable!(),
            }
        );

        let token = Token::ClearFloat { direction };
        state.push_token(token, &*CLEAR_FLOAT);
    }

    Ok(())
}

#[test]
fn test_clear_float() {
    let mut state = ParseState::new("apple\n~~~~\nbanana".into());
    rule_clear_float(&mut state).unwrap();
    assert_eq!(state.text(), "apple\n\00\0\nbanana");
    assert_eq!(state.tokens().len(), 1);

    match state.token(0) {
        Some(Token::ClearFloat { direction }) => {
            assert_eq!(direction.is_none(), true);
        }
        Some(token) => panic!("Token not clear_float, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    let mut state = ParseState::new("durian\n~~~~~~~~\npineapple\n~~~~<\ncherry\n~~~~~>\n".into());
    rule_clear_float(&mut state).unwrap();
    assert_eq!(state.text(), "durian\n\00\0\npineapple\n\01\0\ncherry\n\02\0\n");
    assert_eq!(state.tokens().len(), 3);

    match state.token(0) {
        Some(Token::ClearFloat { direction }) => {
            assert_eq!(direction.is_none(), true);
        }
        Some(token) => panic!("Token not clear_float, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    match state.token(1) {
        Some(Token::ClearFloat { direction }) => {
            assert_eq!(direction.is_some(), true);
            assert_eq!(direction.unwrap(), Direction::Left);
        }
        Some(token) => panic!("Token not clear_float, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    match state.token(2) {
        Some(Token::ClearFloat { direction }) => {
            assert_eq!(direction.is_some(), true);
            assert_eq!(direction.unwrap(), Direction::Right);
        }
        Some(token) => panic!("Token not clear_float, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }
}
