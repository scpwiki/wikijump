/*
 * parse/rules/anchor.rs
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

//! Processing rule for anchors within text.
//! These blocks appear as [[# anchor_name]] and can be
//! linked to as expected of HTML <a> tags.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref ANCHOR: Regex = {
        RegexBuilder::new(r"\[\[# (?P<name>[-_A-Za-z0-9.%]+?)\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_anchor(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = ANCHOR.captures(state.text()) {
        let name = capture["name"].to_string();
        let token = Token::Anchor { name };
        state.push_token(token, &*ANCHOR);
    }

    Ok(())
}

#[test]
fn test_anchor() {
    let mut state = ParseState::new("[[# title]] The Grove of Exiles".into());
    rule_anchor(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0 The Grove of Exiles");
    assert_eq!(state.tokens().len(), 1);
}
