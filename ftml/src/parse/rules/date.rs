/*
 * parse/rules/date.rs
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

//! Processing rule for _TODO_
//! Add documentation!

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref DATE: Regex = {
        RegexBuilder::new(r"\[\[date\s+(?P<timestamp>-?[0-9]+)(\s+.*?)?\]\]")
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_date(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = DATE.captures(state.text()) {
        let timestamp = capture["timestamp"].parse::<i64>().unwrap();
        let args = capture.name("args").map(|mtch| mtch.as_str().to_string());
        let token = Token::Date { timestamp, args };
        state.push_token(token, &*DATE);
    }

    Ok(())
}

#[test]
fn test_date() {
    let mut state = ParseState::new("[[date 0 format=\"%H:%M:%S\"]]\n".into());
    rule_date(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0\n");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("[[date 10000000000]]\n".into());
    rule_date(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0\n");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("[[date -200]]\n".into());
    rule_date(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0\n");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("[[date 200".into());
    rule_date(&mut state).unwrap();
    assert_eq!(state.text(), "[[date 200");
    assert_eq!(state.tokens().len(), 0);
}
