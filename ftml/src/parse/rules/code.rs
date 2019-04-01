/*
 * parse/rules/code.rs
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

//! Processing rule for code blocks.
//! They are multi-line components using [[code]].
//! Currently no syntax highlighting and arguments are ignored.

use crate::{InPlaceReplace, Result};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref CODE_BLOCK: Regex = {
        RegexBuilder::new(r"^\[\[code(?P<args>\s[^\]]*)?\]\](?P<contents>.*?)\[\[/code\]\](?P<end>\s|$)")
            .multi_line(true)
            .dot_matches_new_line(true)
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_code(text: &mut String) -> Result<()> {
    while let Some(capture) = CODE_BLOCK.captures(text) {
        let args = capture.name("args").map(|mtch| mtch.as_str());
        println!("MOCK: rule.code.args {:?}", args);
        // TODO replace with some kind of tree or something
        let replace = format!("<div class=\"code-idk\">{}\n</div>\n{}", &capture["contents"], &capture["end"]);
        text.ireplace_once_regex(&*CODE_BLOCK, &replace);
    }

    Ok(())
}

#[test]
fn test_code() {
    // TODO update when the tree does
    let mut text = String::new();

    text.push_str("[[code]]\nint main() {\n    return 0;\n}[[/code]]");
    rule_code(&mut text).unwrap();
    assert_eq!(&text, "<div class=\"code-idk\">\nint main() {\n    return 0;\n}\n</div>\n");
    text.clear();

    text.push_str("[[code]]\n[[footnote]]Literal footnote code.[[/footnote]]\n[[/code]]");
    rule_code(&mut text).unwrap();
    assert_eq!(&text, "<div class=\"code-idk\">\n[[footnote]]Literal footnote code.[[/footnote]]\n\n</div>\n");
}
