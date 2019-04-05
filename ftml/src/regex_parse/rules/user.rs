/*
 * parse/rules/user.rs
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

//! Processing rule for user references.
//! These blocks take the form of either `[[user rounderhouse]]` or `[[*user rounderhouse]]`.
//! They are replaced with a little clickable reference to the user, combined with their current
//! profile picture (if the `*` is present).

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref USER: Regex = {
        RegexBuilder::new(r"\[\[(?P<modifier>\*?)user (?P<username>[^\]]+)\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_user(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = USER.captures(state.text()) {
        let show_picture = capture["modifier"].len() > 0;
        let username = capture["username"].to_string();
        let token = Token::User { username, show_picture };
        state.push_token(token, &*USER);
    }

    Ok(())
}

#[test]
fn test_user() {
    let mut state = ParseState::new("Written by [[*user rounderhouse]] and [[*user DrAnnoyingDog]]".into());
    rule_user(&mut state).unwrap();
    assert_eq!(state.text(), "Written by \00\0 and \01\0");
    assert_eq!(state.tokens().len(), 2);

    let mut state = ParseState::new("[[*user apple]] + [[user banana]]".into());
    rule_user(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0 + \01\0");
    assert_eq!(state.tokens().len(), 2);

    match state.token(0) {
        Some(Token::User {
            username,
            show_picture,
        }) => {
            assert_eq!(username, "apple");
            assert_eq!(*show_picture, true);
        },
        Some(token) => panic!("Token not user, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    match state.token(1) {
        Some(Token::User {
            username,
            show_picture,
        }) => {
            assert_eq!(username, "banana");
            assert_eq!(*show_picture, false);
        },
        Some(token) => panic!("Token not user, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }
}
