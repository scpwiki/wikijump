/*
 * parse/tree/paragraph/iframe.rs
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

use std::collections::HashMap;
use super::prelude::*;

pub fn parse(pair: Pair<Rule>) -> Paragraph {
    let mut arguments = HashMap::new();
    let mut pairs = pair.into_inner();

    let url = pairs
        .next()
        .expect("Iframe pairs iterator was empty")
        .as_str();

    for pair in pairs {
        debug_assert_eq!(pair.as_rule(), Rule::iframe_arg);

        let key = get_nth_pair!(pair, 0).as_str();
        let value = {
            let pair = get_nth_pair!(pair, 1);
            interp_str(pair.as_str()).expect("Invalid string value")
        };

        arguments.insert(key, value);
    }

    Paragraph::Iframe { url, arguments }
}
