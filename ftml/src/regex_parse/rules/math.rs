/*
 * parse/rules/math.rs
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

//! Processing rule for math blocks.
//! Allows for rendering LaTeX expressions directly in the page.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref MATH: Regex = {
        RegexBuilder::new(r"^\[\[math(?P<label>\s+\w+?)?(?P<args>[^\]]*)?\]\](?P<expr>.*?)\[\[/math\]\](?P<end>\s|$)")
            .multi_line(true)
            .dot_matches_new_line(true)
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_math(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = MATH.captures(state.text()) {
        let label = capture.name("label").map(|mtch| mtch.as_str().to_string());
        let args = capture.name("args").map(|mtch| mtch.as_str().to_string());
        let expr = capture["expr"].to_string();
        let end = capture["end"].to_string();
        let token = Token::Math { label, args, expr, end };
        state.push_token(token, &*MATH);
    }

    Ok(())
}

#[test]
fn test_math() {
    let mut state = ParseState::new("[[math]]\n\\rho(x, y) = x^2 - \\kappa\n[[/math]]\n".into());
    rule_math(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("[[math equation1]]\nf(x) = x\n[[/math]]\n".into());
    rule_math(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("[[math equation2 type=\"align\"]]\na\nb\n[[/math]]\n".into());
    rule_math(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);
}
