/*
 * parse/rules/horizontal.rs
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

//! Processing rule for horizontal ruling lines. What HTML calls `<hr>`.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref HORIZ: Regex = {
        RegexBuilder::new(r"^-{4,}$")
            .multi_line(true)
            .build()
            .unwrap()
    };
}

pub fn rule_horizontal_line(state: &mut ParseState) -> Result<()> {
    while let Some(_) = HORIZ.find(state.text()) {
        let token = Token::HorizontalLine;
        state.push_token(token, &*HORIZ);
    }

    Ok(())
}

#[test]
fn test_horizontal_line() {
    let mut state = ParseState::new("cherry\n----\ndurian".into());
    rule_horizontal_line(&mut state).unwrap();
    assert_eq!(state.text(), "cherry\n\00\0\ndurian");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("banana\n-----\ncherry\n---------\n".into());
    rule_horizontal_line(&mut state).unwrap();
    assert_eq!(state.text(), "banana\n\00\0\ncherry\n\01\0\n");
    assert_eq!(state.tokens().len(), 2);

    let mut state = ParseState::new("apple\n---\nkiwi".into());
    rule_horizontal_line(&mut state).unwrap();
    assert_eq!(state.text(), "apple\n---\nkiwi");
    assert_eq!(state.tokens().len(), 0);
}
