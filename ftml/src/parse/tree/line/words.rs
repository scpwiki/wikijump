/*
 * parse/tree/line/words.rs
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

use crate::enums::HeadingLevel;
use super::prelude::*;

lazy_static! {
    static ref WORDS: Regex = Regex::new(r"^(?P<flag>\+{1,6}|=?)").unwrap();
}

pub fn parse(pair: Pair<Rule>) -> Result<Line> {
    let flag = extract!(WORDS, pair);

    let mut words = Vec::new();
    for pair in pair.into_inner() {
        let word = Word::from_pair(pair)?;
        words.push(word);
    }

    let line = match flag {
        "=" => Line::Words {
            words,
            centered: true,
        },
        "" => Line::Words {
            words,
            centered: false,
        },
        _ => {
            let level = HeadingLevel::try_from(flag.len())
                .expect("Regular expression returned incorrectly-sized heading");

            Line::Heading { words, level }
        }
    };

    Ok(line)
}

#[test]
fn test_regexes() {
    let _ = &*WORDS;
}
