/*
 * parsing/rule/impls/block/blocks/char.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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
use entities::ENTITIES;
use std::borrow::Cow;
use std::collections::HashMap;

lazy_static! {
    static ref ENTITY_MAPPING: HashMap<&str, &str> = {
        let mut mapping = HashMap::new();

        for entity in &ENTITIES {
            let key = strip_entity(entity.entity);
            let value = entity.characters;

            mapping.insert(key, value);
        }

        mapping
    };
}

pub const BLOCK_CHAR: BlockRule = BlockRule {
    name: "block-char",
    accepts_names: &["char", "character"],
    accepts_special: false,
    accepts_newlines: false,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    special: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Parsing character / HTML entity block"; "in-head" => in_head);

    assert_eq!(special, false, "Char doesn't allow special variant");
    assert_block_name(&BLOCK_CHAR, name);

    let entity = parser.get_head_value(&BLOCK_CHAR, in_head, parse_count)?;
    let result = find_entity(strip_entity(entity));

    ok!(Element::Text(result))
}

/// Find the string corresponding to the passed entity, if any.
fn find_entity(entity: &str) -> Option<Cow<str>> {
    // Named entity
    if let Some(result) = ENTITY_MAPPING.get(entity) {
        return Some(cow!(result));
    }

    // Hexadecimal entity
    if entity.starts_with("#x") {
        if let Some(result) = get_char(&entity[2..], 16) {
            return Some(result);
        }
    }

    // Decimal entity
    if entity.starts_with('#') {
        if let Some(result) = get_char(&entity[1..], 10) {
            return Some(result);
        }
    }

    // Not found
    None
}

/// Gets the appropriate character from the number specified in the string.
///
/// Using the passed radix, it gets the integer value, then finds the appropriate
/// character, if one exists.
///
/// Then converts the character into a string with only that value.
fn get_char(value: &str, radix: u32) -> Option<Cow<str>> {
    let codepoint = match u32::from_str_radix(value, radix) {
        Ok(codepoint) => codepoint,
        Err(_) => return None,
    };

    let ch = match char::from_u32(codepoint) {
        Some(ch) => ch,
        None => return None,
    };

    Some(Cow::Borrowed(ch.to_string()))
}

/// If a string starts with `&` and ends with `;`, those are removed.
fn strip_entity(s: &str) -> &str {
    if s.starts_with('&') && s.ends_with(';') {
        // Strip first and last characters

        &s[1..s.len() - 1]
    } else {
        // Leave unchanged

        s
    }
}

#[test]
fn test_strip_entity() {
    macro_rules! check {
        ($input:expr, $expected:expr $(,)?) => {{
            let actual = strip_entity($input);
            let expected = $expected;

            assert_eq!(
                actual,
                expected,
                "Actual stripped entity value didn't match expected",
            );
        }};
    }

    check!("", "");
    check!("abc", "abc");
    check!("legumes1", "legumes1");
    check!("&amp;", "amp");
    check!("&#100;", "#100");
    check!("&xdeadbeef;", "xdeadbeef");
}
