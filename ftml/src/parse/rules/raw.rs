/*
 * parse/rules/raw.rs
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

//! Processing rule for raw text. Raw snippets ignore formatting and other
//! parsing, allowing for literals which would otherwise cause syntax issues.
//!
//! Also allows you to use the legacy raw style. Hopefully we can phase that
//! out very soon!

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref RAW: Regex = {
        RegexBuilder::new(r"@@(?P<contents>.*?[^@]?)@@")
            .build()
            .unwrap()
    };

    static ref RAW_OLD: Regex = {
        RegexBuilder::new(r"``(?P<contents>[^']*?[^`])``")
            .build()
            .unwrap()
    };
}

const SUPPORT_LEGACY_RAW: bool = true;

pub fn rule_raw(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = RAW.captures(state.text()) {
        let contents = capture["contents"].to_string();
        let token = Token::Raw { contents };
        state.push_token(token, &*RAW);
    }

    if SUPPORT_LEGACY_RAW {
        while let Some(capture) = RAW_OLD.captures(state.text()) {
            let contents = capture["contents"].to_string();
            let token = Token::Raw { contents };
            state.push_token(token, &*RAW_OLD);
        }
    }

    Ok(())
}

#[test]
fn test_raw() {
    let mut state = ParseState::new("@@ [[code]] @@ @@@@".into());
    rule_raw(&mut state).unwrap();
    assert_eq!(state.text(), "\0 \0");

    match state.token(0) {
        Some(Token::Raw { contents }) => assert_eq!(contents, " [[code]] "),
        Some(token) => panic!("Token not raw, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    match state.token(1) {
        Some(Token::Raw { contents }) => assert_eq!(contents, ""),
        Some(token) => panic!("Token not raw, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    let mut state = ParseState::new("`` {{apple}} ``".into());
    rule_raw(&mut state).unwrap();

    if SUPPORT_LEGACY_RAW {
        assert_eq!(state.text(), "\0");
        assert_eq!(state.tokens().len(), 1);
    } else {
        assert_eq!(state.text(), "`` {{apple}} ``");
        assert_eq!(state.tokens().len(), 0);
    }
}
