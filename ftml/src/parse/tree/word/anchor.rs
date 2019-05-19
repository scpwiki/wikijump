/*
 * parse/tree/word/anchor.rs
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

use crate::enums::AnchorTarget;
use super::prelude::*;

#[derive(Debug, Default)]
struct Context<'a> {
    href: Option<&'a str>,
    name: Option<&'a str>,
    id: Option<&'a str>,
    class: Option<&'a str>,
    style: Option<&'a str>,
    target: Option<AnchorTarget>,
    words: Vec<Word<'a>>,
}

pub fn parse(pair: Pair<Rule>) -> Result<Word> {
    let mut ctx = Context::default();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::anchor_arg => parse_arg(&mut ctx, pair),
            Rule::word => {
                let word = Word::from_pair(pair)?;
                ctx.words.push(word);
            }
            _ => panic!("Invalid rule for anchor: {:?}", pair.as_rule()),
        }
    }

    let Context {
        href,
        name,
        id,
        class,
        style,
        target,
        words,
    } = ctx;
    Ok(Word::Anchor {
        href,
        name,
        id,
        class,
        target,
        style,
        words,
    })
}

fn parse_arg<'c, 'p>(ctx: &'c mut Context<'p>, pair: Pair<'p, Rule>) {
    let capture = ARGUMENT_NAME
        .captures(pair.as_str())
        .expect("Regular expression ARGUMENT_NAME didn't match");
    let key = capture!(capture, "name");
    let value_pair = get_first_pair!(pair);

    debug_assert_eq!(value_pair.as_rule(), Rule::string);

    let value = value_pair.as_str();
    match key.to_ascii_lowercase().as_str() {
        "href" => ctx.href = Some(value),
        "name" => ctx.name = Some(value),
        "id" => ctx.id = Some(value),
        "class" => ctx.class = Some(value),
        "style" => ctx.style = Some(value),
        "target" => ctx.target = AnchorTarget::try_from(value).ok(),
        _ => panic!("Unknown argument for [[a]]: {}", key),
    }
}
