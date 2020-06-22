/*
 * parse/tree/word/anchor.rs
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
use crate::enums::AnchorTarget;

#[derive(Debug, Default)]
struct Context<'a> {
    href: Option<Cow<'a, str>>,
    name: Option<Cow<'a, str>>,
    id: Option<Cow<'a, str>>,
    class: Option<Cow<'a, str>>,
    style: Option<Cow<'a, str>>,
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

    // Helper data to set the appropriate field based on key
    #[derive(Debug, Copy, Clone)]
    enum Field {
        Href,
        Name,
        Id,
        Class,
        Style,
        Target,
    }

    const ANCHOR_VALUES: [(&str, Field); 6] = [
        ("href", Field::Href),
        ("name", Field::Name),
        ("id", Field::Id),
        ("class", Field::Class),
        ("style", Field::Style),
        ("target", Field::Target),
    ];

    fn get_field(key: &str) -> Field {
        for (name, field) in &ANCHOR_VALUES {
            if key.eq_ignore_ascii_case(name) {
                return *field;
            }
        }

        panic!("Unknown argument for [[a]]: {}", key);
    }

    let value = interp_str(value_pair.as_str()).expect("Invalid string value");
    match get_field(key) {
        Field::Href => ctx.href = Some(value),
        Field::Name => ctx.name = Some(value),
        Field::Id => ctx.id = Some(value),
        Field::Class => ctx.class = Some(value),
        Field::Style => ctx.style = Some(value),
        Field::Target => ctx.target = AnchorTarget::try_from(value.as_ref()).ok(),
    }
}
