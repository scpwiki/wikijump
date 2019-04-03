/*
 * parse/rules/div_prefilter.rs
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

//! Pre-filtering processing rule for divs.
//! It collapses spaces between `[[div]]` tags. I'm not really sure why Wikidot added it.

use crate::{ParseState, Result, Token};
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref DIV_PREFILTER: Regex = {
        RegexBuilder::new(r"\[\[/div\]\]\s+\[\[div")
            .multi_line(true)
            .dot_matches_new_line(true)
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}

pub fn rule_div_prefilter(state: &mut ParseState) -> Result<()> {
    state.replace_all_regex(&*DIV_PREFILTER, "[[/div]][[div");
    Ok(())
}

#[test]
fn test_div_prefilter() {
    let mut state = ParseState::new("[[/div]]    [[div class=\"tm-Viewer\"]]".into());
    rule_div_prefilter(&mut state).unwrap();
    assert_eq!(state.text(), "[[/div]][[div class=\"tm-Viewer\"]]");
}
