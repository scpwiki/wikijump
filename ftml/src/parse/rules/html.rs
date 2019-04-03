/*
 * parse/rules/html.rs
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

//! Processing rule for inline HTML.
//! These are multi-line components using `[[html]]` to embed literal HTML.
//! Contrary to what the source code suggests, Wikidot does _not_ support embedding
//! HTML using `<html>` tags.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref HTML: Regex = {
        RegexBuilder::new(r"^\[\[html\]\]\n(?P<contents>.*?)\[\[/html\]\]")
            .multi_line(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
}

pub fn rule_html(state: &mut ParseState) -> Result<()> {
    while let Some(capture) = HTML.captures(state.text()) {
        let contents = capture["contents"].to_string();
        let token = Token::Html { contents };
        state.push_token(token, &*HTML);
    }

    Ok(())
}

#[test]
fn test_html() {
    let mut state = ParseState::new("[[html]]\n<span>test</span>\n[[/html]]".into());
    rule_html(&mut state).unwrap();
    assert_eq!(state.text(), "\00\0");
    assert_eq!(state.tokens().len(), 1);

    let mut state = ParseState::new("<html>\n<span>test</span>\n</html>".into());
    rule_html(&mut state).unwrap();
    assert_eq!(state.text(), "<html>\n<span>test</span>\n</html>");
    assert_eq!(state.tokens().len(), 0);
}
