/*
 * parsing/rule/impls/block/blocks/char.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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
use std::char;
use std::collections::HashMap;
use std::sync::LazyLock;

static ENTITY_MAPPING: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut mapping = HashMap::new();

        for entity in &ENTITIES {
            let key = strip_entity(entity.entity);
            let value = entity.characters;

            mapping.insert(key, value);
        }

        mapping
    });

pub const BLOCK_CHAR: BlockRule = BlockRule {
    name: "block-char",
    accepts_names: &["char", "character"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: false,
    parse_fn,
};

fn parse_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing character / HTML entity block (in-head {in_head})");
    assert!(!flag_star, "Char doesn't allow star flag");
    assert!(!flag_score, "Char doesn't allow score flag");
    assert_block_name(&BLOCK_CHAR, name);

    // Parse the entity and get the string
    let string = parser.get_head_value(&BLOCK_CHAR, in_head, parse_entity)?;

    ok!(Element::Text(string))
}

fn parse_entity<'t>(
    parser: &Parser<'_, 't>,
    argument: Option<&'t str>,
) -> Result<Cow<'t, str>, ParseError> {
    let argument = match argument {
        Some(arg) => strip_entity(arg),
        None => return Err(parser.make_err(ParseErrorKind::BlockMissingArguments)),
    };

    match find_entity(argument) {
        Some(string) => Ok(string),
        None => Err(parser.make_err(ParseErrorKind::BlockMalformedArguments)),
    }
}

/// Find the string corresponding to the passed entity, if any.
fn find_entity(entity: &str) -> Option<Cow<'_, str>> {
    // Named entity
    if let Some(result) = ENTITY_MAPPING.get(entity) {
        return Some(cow!(result));
    }

    // Hexadecimal entity
    if let Some(value) = entity.strip_prefix("#x")
        && let Some(result) = get_char(value, 16)
    {
        return Some(result);
    }

    // Decimal entity
    if let Some(value) = entity.strip_prefix('#')
        && let Some(result) = get_char(value, 10)
    {
        return Some(result);
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
fn get_char(value: &str, radix: u32) -> Option<Cow<'_, str>> {
    let codepoint = match u32::from_str_radix(value, radix) {
        Ok(codepoint) => codepoint,
        Err(_) => return None,
    };

    let ch = char::from_u32(codepoint)?;
    Some(Cow::Owned(ch.to_string()))
}

/// If a string starts with `&` or ends with `;`, those are removed.
/// First trims the string of whitespace.
fn strip_entity(mut s: &str) -> &str {
    s = s.trim();

    if let Some(stripped) = s.strip_prefix('&') {
        s = stripped;
    }

    if let Some(stripped) = s.strip_suffix(';') {
        s = stripped;
    }

    s
}

/* Tests */

#[test]
fn test_get_entity() {
    macro_rules! test {
        ($input:expr, $expected:expr $(,)?) => {{
            let actual = find_entity($input);
            let expected = $expected;

            assert_eq!(
                actual, expected,
                "Actual entity string doesn't match expected",
            );
        }};
    }

    test!("", None);

    // Names
    test!("amp", Some(cow!("&")));
    test!("lt", Some(cow!("<")));
    test!("gt", Some(cow!(">")));
    test!("copy", Some(cow!("©")));
    test!("xxxzzz", None);

    // Decimal
    test!("#32", Some(cow!(" ")));
    test!("#255", Some(cow!("\u{ff}")));
    test!("#128175", Some(cow!("💯")));
    test!("#2097151", None);

    // Hex
    test!("#x20", Some(cow!(" ")));
    test!("#xff", Some(cow!("\u{ff}")));
    test!("#x1f4af", Some(cow!("💯")));
    test!("#x1fffff", None);
}

#[test]
fn test_get_char() {
    macro_rules! test {
        ($value:expr, $radix:expr, $expected:expr $(,)?) => {{
            let actual = get_char($value, $radix);
            let expected = $expected;

            assert_eq!(
                actual, expected,
                "Actual character value doesn't match expected",
            );
        }};
    }

    // Decimal
    test!("32", 10, Some(Cow::Owned(str!(' '))));
    test!("255", 10, Some(Cow::Owned(str!('\u{ff}'))));
    test!("128175", 10, Some(Cow::Owned(str!('💯'))));
    test!("2097151", 10, None);

    // Hex
    test!("20", 16, Some(Cow::Owned(str!(' '))));
    test!("ff", 16, Some(Cow::Owned(str!('\u{ff}'))));
    test!("1f4af", 16, Some(Cow::Owned(str!('💯'))));
    test!("1fffff", 16, None);
}

#[test]
fn test_strip_entity() {
    macro_rules! test {
        ($input:expr, $expected:expr $(,)?) => {{
            let actual = strip_entity($input);
            let expected = $expected;

            assert_eq!(
                actual, expected,
                "Actual stripped entity value didn't match expected",
            );
        }};
    }

    test!("", "");
    test!("abc", "abc");
    test!("legumes1", "legumes1");
    test!("&amp;", "amp");
    test!("&#100;", "#100");
    test!("&xdeadbeef;", "xdeadbeef");

    test!("&amp", "amp");
    test!("amp;", "amp");
    test!("&#100", "#100");
    test!("#100;", "#100");

    test!(" ", "");
    test!(" abc", "abc");
    test!(" legumes1", "legumes1");
    test!(" &amp;", "amp");
    test!(" &#100;", "#100");
    test!(" &xdeadbeef;", "xdeadbeef");
}
