/*
 * parse/rules/code.rs
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

//! Processing rule for code blocks.
//! They are multi-line components using [[code]].
//! Currently no syntax highlighting and arguments are ignored.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref CODE_BLOCK: Regex = {
        RegexBuilder::new(r"^\[\[code(?P<args>\s[^\]]*)?\]\](?P<contents>.*?)\[\[/code\]\](?P<end>\s|$)")
            .multi_line(true)
            .dot_matches_new_line(true)
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_code(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = CODE_BLOCK.captures(state.text()) {
        let args = capture.name("args").map(|mtch| mtch.as_str().to_string());
        let contents = capture["contents"].to_string();
        let token = Token::CodeBlock { args, contents };
        let replace_with = format!("\0{}", &capture["end"]);
        state.replace_once_regex(&*CODE_BLOCK, &replace_with);
        state.push_token(token);
    }

    Ok(())
}

#[test]
fn test_code() {
    let mut state = ParseState::new("[[code]]\nif condition:\n    print('hi')\n[[/code]]\n".into());
    rule_code(&mut state).unwrap();
    assert_eq!(state.text(), "\0\n");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("[[code]]\nincomplete".into());
    rule_code(&mut state).unwrap();
    assert_eq!(state.text(), "[[code]]\nincomplete");
    assert_eq!(state.tokens().len(), 0);

    let mut state = ParseState::new(
        "Apple\n[[code]]\nBanana\n[[/code]]\nCherry\n[[code args=value]]\nDurian\n[[/code]]\n"
            .into(),
    );
    rule_code(&mut state).unwrap();
    assert_eq!(state.text(), "Apple\n\0\nCherry\n\0\n");
    assert_eq!(state.tokens().len(), 2);
}
