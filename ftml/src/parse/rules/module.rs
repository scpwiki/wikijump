/*
 * parse/rules/module.rs
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

//! Processing rule for Wikidot modules.
//! We don't currently have any of them implemented though!
//!
//! Also we don't support module654 or the other weird stuff
//! in the Wikidot code. There aren't any docs on them, and
//! nobody seems to use them, and where they _are_ used they
//! don't seem to be functional.
//! What they're even for I have no clue.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref MODULE: Regex = {
        RegexBuilder::new(r"^\[\[module\s(?P<name>[a-z0-9_\-/]+)(?P<args>\s+.*?)?\]\]\n(?:(?P<contents>.*?)\[\[/module\]\])?")
            .multi_line(true)
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
}

pub fn rule_module(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = MODULE.captures(state.text()) {
        let name = capture["name"].to_string();
        let args = capture.name("args").map(|mtch| mtch.as_str().to_string());
        let contents = capture.name("contents").map(|mtch| mtch.as_str().to_string());
        let token = Token::Module { name, args, contents };
        state.replace_once_regex(&*MODULE, "\0");
        state.push_token(token);
    }

    Ok(())
}

#[test]
fn test_module() {
    let mut state = ParseState::new("[[module]]\n[[/module]]".into());
    rule_module(&mut state).unwrap();
    assert_eq!(state.text(), "[[module]]\n[[/module]]");

    let mut state = ParseState::new("[[module Rate]]\n[[/module]]\nbanana".into());
    rule_module(&mut state).unwrap();
    assert_eq!(state.text(), "\0\nbanana");

    match state.token(0) {
        Some(Token::Module { name, args, contents }) => {
            assert_eq!(name, "Rate");
            assert!(args.is_none());
            assert_eq!(contents.as_ref().unwrap(), "");
        },
        Some(token) => panic!("Token not raw, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }

    let mut state = ParseState::new("apple\n[[module ListPages category=\"fragment\"]]\n%%content%%\n[[/module]]".into());
    rule_module(&mut state).unwrap();
    assert_eq!(state.text(), "apple\n\0");

    match state.token(0) {
        Some(Token::Module { name, args, contents }) => {
            assert_eq!(name, "ListPages");
            assert_eq!(args.as_ref().unwrap(), " category=\"fragment\"");
            assert_eq!(contents.as_ref().unwrap(), "%%content%%\n");
        },
        Some(token) => panic!("Token not raw, was {:?}", token),
        None => panic!("Not enough tokens in state"),
    }
}
