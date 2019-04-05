/*
 * parse/rules/comment.rs
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

//! Processing rule for comments.

use crate::{ParseState, Result};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref COMMENT: Regex = {
        RegexBuilder::new(r"\n?\[!--(.*?)--\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
}

pub fn rule_comment(state: &mut ParseState) -> Result<()> {
    state.replace_all_regex(&*COMMENT, "");
    Ok(())
}

#[test]
fn test_comment() {
    let mut state = ParseState::new("hello [!-- apple --] world!".into());
    rule_comment(&mut state).unwrap();
    assert_eq!(state.text(), "hello  world!");

    let mut state = ParseState::new("[!-- apple\n cherry \n --] grapefruit".into());
    rule_comment(&mut state).unwrap();
    assert_eq!(state.text(), " grapefruit");
}
