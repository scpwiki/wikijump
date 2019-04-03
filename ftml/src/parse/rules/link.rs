/*
 * parse/rules/link.rs
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

//! Processing rule for
//! Add documentation!

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref LINK: Regex = {
        RegexBuilder::new(r"\[\[\[(?P<page>[^\]\|\[#]+)\s*(?P<anchor>#[a-z][a-z0-9\-_:.]*)?\s*(?P<text>\|[^\]\|\[#]*)?\]\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_link(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = LINK.captures(state.text()) {
        let page = capture["page"].to_string();
        let anchor = capture.name("anchor").map(|mtch| mtch.as_str().to_string());
        let text = capture.name("text").map(|mtch| mtch.as_str().to_string());
        let token = Token::Link { page, anchor, text };
        state.push_token(token, &*LINK);
    }

    Ok(())
}

#[test]
fn test_link() {
    let mut state = ParseState::new("[[[SCP-001]]]".into());
    rule_link(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("the [[[SCP-4002 | Black Moon]]]".into());
    rule_link(&mut state).unwrap();
    assert_eq!(state.text(), "the \00\0");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("[[[page-title|]]] cherry".into());
    rule_link(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0 cherry");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("Durians: [[[doc#toc1|Section 1]]]\n".into());
    rule_link(&mut state).unwrap();
    assert_eq!(state.text(), "Durians: \00\0\n");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("[[[/|Root]]]\n".into());
    rule_link(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0\n");
    assert_eq!(state.tokens().len(), 1);
}
