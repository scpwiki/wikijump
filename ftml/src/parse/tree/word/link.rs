/*
 * parse/tree/word/link.rs
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

use super::prelude::*;

pub fn parse_bare(pair: Pair<Rule>) -> Word {
    let mut pairs = pair.into_inner();

    let target =
        get_link_target(pairs.next().expect("LinkBare pairs iterator was empty"));

    let href = pairs
        .next()
        .expect("LinkBare pairs iterator had only one element")
        .as_str();

    Word::Link {
        href,
        target,
        text: None,
    }
}

pub fn parse_page(pair: Pair<Rule>) -> Word {
    let mut pairs = pair.into_inner();

    let target =
        get_link_target(pairs.next().expect("LinkPage pairs iterator was empty"));

    let href = pairs
        .next()
        .expect("LinkPage pairs iterator had only one element")
        .as_str();

    let use_page_title = pairs.next().is_some();
    let text = if use_page_title {
        pairs.next().map(|pair| pair.as_str())
    } else {
        Some("")
    };

    Word::Link { href, target, text }
}

pub fn parse_url(pair: Pair<Rule>) -> Word {
    let mut pairs = pair.into_inner();

    let target =
        get_link_target(pairs.next().expect("LinkUrl pairs iterator was empty"));

    let href = pairs
        .next()
        .expect("LinkUrl pairs iterator had only one element");

    let text = pairs
        .next()
        .expect("LinkUrl pairs iterator had only two elements")
        .as_str();
    let text = Some(text);

    match href.as_rule() {
        Rule::email => Word::Email {
            address: href.as_str(),
            text,
        },
        Rule::link_url_href => Word::Link {
            href: href.as_str(),
            target,
            text,
        },
        _ => panic!("Invalid rule for link_url: {:?}", href.as_rule()),
    }
}
