/*
 * parse/rules/form.rs
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

//! Processing rule for Data Forms. In Wikidot, these are a way to template
//! different categories of pages, but we don't really use them so we're just
//! going to add a dummy placeholder.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref FORM: Regex = {
        RegexBuilder::new(r"\[\[form\]\]\n(?P<contents>.*)\[\[/form\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
}

pub fn rule_form(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = FORM.captures(state.text()) {
        let contents = capture["contents"].to_string();
        let token = Token::Form { contents };
        state.push_token(token, &*FORM);
    }

    Ok(())
}

#[test]
fn test_form() {
    let mut state =
        ParseState::new("apple\n[[form]]\nuseless feature tbh\n[[/form]]\nblueberry".into());
    rule_form(&mut state).unwrap();
    assert_eq!(state.text(), "apple\n\00\0\nblueberry");
    assert_eq!(state.tokens().len(), 1);
}
