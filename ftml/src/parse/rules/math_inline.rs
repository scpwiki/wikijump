/*
 * parse/rules/math_inline.rs
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

//! Processing rule for inline math blocks.
//! Allows for rendering LaTeX expressions directly in the page.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref MATH_INLINE: Regex = {
        RegexBuilder::new(r"\[\[\$(?P<expr>.*?)\$\]\]")
            .build()
            .unwrap()
    };
}

pub fn rule_math_inline(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = MATH_INLINE.captures(state.text()) {
        let expr = capture["expr"].to_string();
        let token = Token::MathInline { expr };
        state.push_token(token, &*MATH_INLINE);
    }

    Ok(())
}

#[test]
fn test_math_inline() {
    let mut state = ParseState::new("[[$ \\rho(x, y) = x^2 - \\kappa $]]\n".into());
    rule_math_inline(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0\n");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("apple [[$ f(x) $]] banana".into());
    rule_math_inline(&mut state).unwrap();
    assert_eq!(state.text(), "apple \00\0 banana");
    assert_eq!(state.tokens().len(), 1);
}
