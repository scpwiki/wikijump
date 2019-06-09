/*
 * parse/tree/paragraph/align.rs
 *
 * ftml - Convert Wikidot code to HTML
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

use crate::enums::Alignment;
use super::prelude::*;

lazy_static! {
    static ref ALIGN: Regex = Regex::new(r"^\[\[(?P<direction><|>|=|==)\]\]").unwrap();
}

pub fn parse(pair: Pair<Rule>) -> Result<Paragraph> {
    let alignment = Alignment::try_from(extract!(ALIGN, pair))
        .expect("Parsed align block had invalid alignment");

    let result: Result<Vec<_>> = pair.into_inner().map(Paragraph::from_pair).collect();
    let paragraphs = result?;

    Ok(Paragraph::Align { alignment, paragraphs })
}

#[test]
fn test_regexes() {
    let _ = &*ALIGN;
}
