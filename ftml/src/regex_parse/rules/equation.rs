/*
 * parse/rules/equation.rs
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

//! Processing rule for equation references.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref EQUATION_REF: Regex = {
        RegexBuilder::new(r"\[\[eref\s+(?P<label>.+?)\]\]")
            .build()
            .unwrap()
    };
}

pub fn rule_equation(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = EQUATION_REF.captures(state.text()) {
        let label = capture["label"].to_string();
        let token = Token::Equation { label };
        state.push_token(token, &*EQUATION_REF);
    }

    Ok(())
}

#[test]
fn test_equation() {
    let mut state = ParseState::new("[[eref]]".into());
    rule_equation(&mut state).unwrap();
    assert_eq!(state.text(), "[[eref]]");
    assert_eq!(state.tokens().len(), 0);

    let mut state = ParseState::new("[[eref equation1]]".into());
    rule_equation(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);
}
