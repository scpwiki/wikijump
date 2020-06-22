/*
 * parse/tree/paragraph/list.rs
 *
 * ftml - Library to parse Wikidot code
 * Copyright (C) 2019-2020 Ammon Smith
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
use crate::enums::ListStyle;

pub fn parse(pair: Pair<Rule>) -> Result<Paragraph> {
    let depth = {
        let mut depth = 0;
        for ch in pair.as_str().chars() {
            match ch {
                ' ' => depth += 1,
                _ => break,
            }
        }
        depth
    };

    let style = match pair.as_rule() {
        Rule::bullet_list => ListStyle::Bullet,
        Rule::numbered_list => ListStyle::Numbered,
        _ => unreachable!(),
    };

    let mut items = Vec::new();
    for pair in pair.into_inner() {
        debug_assert_eq!(pair.as_rule(), Rule::list_item);

        let mut words = Vec::new();
        for pair in pair.into_inner() {
            let word = Word::from_pair(pair)?;
            words.push(word);
        }

        let paragraph = Paragraph::Words {
            words,
            centered: false,
        };
        items.push(paragraph);
    }

    Ok(Paragraph::List {
        style,
        depth,
        items,
    })
}
