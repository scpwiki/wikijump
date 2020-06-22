/*
 * parse/tree/word/link.rs
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

pub fn parse_bare(pair: Pair<Rule>) -> Word {
    let mut pairs = pair.into_inner();

    let target = get_link_target(pairs.next().expect("LinkBare pairs iterator was empty"));

    let href = pairs
        .next()
        .expect("LinkBare pairs iterator had only one element")
        .as_str();

    Word::Link {
        href,
        target,
        text: LinkText::Url,
    }
}

pub fn parse_page(pair: Pair<Rule>) -> Word {
    let mut pairs = pair.into_inner();

    let target = get_link_target(pairs.next().expect("LinkPage pairs iterator was empty"));

    let href = pairs
        .next()
        .expect("LinkPage pairs iterator had only one element")
        .as_str();

    let use_link_name = pairs.next().is_some();
    let text = if use_link_name {
        match pairs.next() {
            Some(pair) => LinkText::Text(pair.as_str()),
            None => LinkText::Article,
        }
    } else {
        LinkText::Url
    };

    Word::Link { href, target, text }
}

pub fn parse_url(pair: Pair<Rule>) -> Word {
    let mut pairs = pair.into_inner();

    let target = get_link_target(pairs.next().expect("LinkUrl pairs iterator was empty"));

    let href = pairs
        .next()
        .expect("LinkUrl pairs iterator had only one element");

    let text = pairs
        .next()
        .expect("LinkUrl pairs iterator had only two elements")
        .as_str();

    match href.as_rule() {
        Rule::email => Word::Email {
            address: href.as_str(),
            text: Some(text),
        },
        Rule::link_url_href => Word::Link {
            target,
            href: href.as_str(),
            text: LinkText::Text(text),
        },
        _ => panic!("Invalid rule for link_url: {:?}", href.as_rule()),
    }
}

pub fn parse_file(pair: Pair<Rule>) -> Word {
    let mut pairs = pair.into_inner();

    let target = get_link_target(pairs.next().expect("FileRef pairs iterator was empty"));

    let filename = pairs
        .next()
        .expect("FileRef pairs iterator had only one element")
        .as_str();

    let text = pairs.next().map(|p| p.as_str());

    Word::File {
        filename,
        text,
        target,
    }
}
