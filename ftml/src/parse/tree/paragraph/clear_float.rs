/*
 * parse/tree/paragraph/clear_float.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

use super::prelude::*;
use crate::enums::Alignment;

lazy_static! {
    static ref CLEAR_FLOAT: Regex = Regex::new(r"~{4,}(?P<direction><|>|=)?").unwrap();
}

pub fn parse(pair: Pair<Rule>) -> Paragraph {
    let capture = CLEAR_FLOAT
        .captures(pair.as_str())
        .expect("Regular expression CLEAR_FLOAT didn't match");

    let direction = match capture.name("direction") {
        Some(mtch) => Some(
            Alignment::try_from(mtch.as_str())
                .ok()
                .expect("Alignment conversion failed"),
        ),
        None => None,
    };

    Paragraph::ClearFloat { direction }
}

#[test]
fn test_regexes() {
    let _ = &*CLEAR_FLOAT;
}
