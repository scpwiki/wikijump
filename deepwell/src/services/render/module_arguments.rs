/*
 * services/render/module_arguments.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

//! Parsing for argument heads shared by Wikidot runtime modules.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::services::render) struct WikidotModuleArgument<'a> {
    pub(in crate::services::render) key: &'a str,
    pub(in crate::services::render) op: &'a str,
    pub(in crate::services::render) value: &'a str,
}

pub(in crate::services::render) fn module_arguments_are_complete(head: &str) -> bool {
    wikidot_module_arguments(head).is_some()
}

pub(in crate::services::render) fn wikidot_module_argument<'a>(
    head: &'a str,
    name: &str,
) -> Option<&'a str> {
    wikidot_module_arguments(head)?
        .into_iter()
        .rev()
        .find(|argument| argument.key.eq_ignore_ascii_case(name))
        .map(|argument| argument.value)
}

pub(in crate::services::render) fn wikidot_module_arguments(
    head: &str,
) -> Option<Vec<WikidotModuleArgument<'_>>> {
    let mut arguments = Vec::new();
    let mut cursor = 0usize;
    skip_wikidot_argument_whitespace(head, &mut cursor);

    while cursor < head.len() {
        let key_start = cursor;
        while head.as_bytes().get(cursor).is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
        }) {
            cursor += 1;
        }
        if cursor == key_start {
            return None;
        }
        let key = &head[key_start..cursor];
        let first_key_byte = key.as_bytes()[0];
        if !(first_key_byte.is_ascii_alphabetic() || first_key_byte == b'_') {
            return None;
        }

        skip_wikidot_argument_whitespace(head, &mut cursor);
        let op_start = cursor;
        if head.as_bytes().get(cursor) == Some(&b'!') {
            cursor += 1;
        }
        if head.as_bytes().get(cursor) != Some(&b'=') {
            return None;
        }
        cursor += 1;
        let op = &head[op_start..cursor];
        skip_wikidot_argument_whitespace(head, &mut cursor);
        if cursor >= head.len() {
            return None;
        }

        let value_start = cursor;
        let first = head[value_start..].chars().next()?;
        let value = if first == '"' {
            match wikidot_double_quoted_argument_value(head, value_start) {
                Some((value, next)) => {
                    cursor = next;
                    value
                }
                None => {
                    cursor = wikidot_bare_argument_end(head, value_start);
                    &head[value_start..cursor]
                }
            }
        } else if first == '\'' {
            match wikidot_single_quoted_argument_value(head, value_start) {
                Some((value, next)) => {
                    cursor = next;
                    value
                }
                None => {
                    cursor = wikidot_bare_argument_end(head, value_start);
                    &head[value_start..cursor]
                }
            }
        } else {
            cursor = wikidot_bare_argument_end(head, value_start);
            if cursor == value_start {
                return None;
            }
            &head[value_start..cursor]
        };

        arguments.push(WikidotModuleArgument { key, op, value });
        skip_wikidot_argument_whitespace(head, &mut cursor);
    }

    Some(arguments)
}

pub(in crate::services::render) fn wikidot_module_arguments_ignoring_bare_flags(
    head: &str,
) -> Option<Vec<WikidotModuleArgument<'_>>> {
    let mut arguments = Vec::new();
    let mut cursor = 0usize;
    skip_wikidot_argument_whitespace(head, &mut cursor);

    while cursor < head.len() {
        let key_start = cursor;
        while head.as_bytes().get(cursor).is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
        }) {
            cursor += 1;
        }
        if cursor == key_start {
            return None;
        }
        let key = &head[key_start..cursor];
        let first_key_byte = key.as_bytes()[0];
        if !(first_key_byte.is_ascii_alphabetic() || first_key_byte == b'_') {
            return None;
        }

        let cursor_after_key = cursor;
        skip_wikidot_argument_whitespace(head, &mut cursor);
        let op_start = cursor;
        if head.as_bytes().get(cursor) == Some(&b'!') {
            cursor += 1;
        }
        if head.as_bytes().get(cursor) != Some(&b'=') {
            if cursor > cursor_after_key || cursor >= head.len() {
                continue;
            }
            cursor = wikidot_bare_argument_end(head, cursor);
            skip_wikidot_argument_whitespace(head, &mut cursor);
            continue;
        }
        cursor += 1;
        let op = &head[op_start..cursor];
        skip_wikidot_argument_whitespace(head, &mut cursor);
        if cursor >= head.len() {
            return None;
        }

        let value_start = cursor;
        let first = head[value_start..].chars().next()?;
        let value = if first == '"' {
            match wikidot_double_quoted_argument_value(head, value_start) {
                Some((value, next)) => {
                    cursor = next;
                    value
                }
                None => {
                    cursor = wikidot_bare_argument_end(head, value_start);
                    &head[value_start..cursor]
                }
            }
        } else if first == '\'' {
            match wikidot_single_quoted_argument_value(head, value_start) {
                Some((value, next)) => {
                    cursor = next;
                    value
                }
                None => {
                    cursor = wikidot_bare_argument_end(head, value_start);
                    &head[value_start..cursor]
                }
            }
        } else {
            cursor = wikidot_bare_argument_end(head, value_start);
            if cursor == value_start {
                return None;
            }
            &head[value_start..cursor]
        };

        arguments.push(WikidotModuleArgument { key, op, value });
        skip_wikidot_argument_whitespace(head, &mut cursor);
    }

    Some(arguments)
}

fn wikidot_double_quoted_argument_value(
    head: &str,
    quote_start: usize,
) -> Option<(&str, usize)> {
    let mut cursor = quote_start + '"'.len_utf8();
    while cursor < head.len() {
        let character = head[cursor..].chars().next()?;
        if character == '"' && wikidot_argument_boundary_at(head, cursor + 1) {
            return Some((&head[quote_start + 1..cursor], cursor + 1));
        }
        cursor += character.len_utf8();
    }
    None
}

fn wikidot_single_quoted_argument_value(
    head: &str,
    quote_start: usize,
) -> Option<(&str, usize)> {
    let mut cursor = quote_start + '\''.len_utf8();
    while cursor < head.len() {
        let character = head[cursor..].chars().next()?;
        if character == '\'' {
            return Some((&head[quote_start + 1..cursor], cursor + 1));
        }
        cursor += character.len_utf8();
    }
    None
}

fn wikidot_bare_argument_end(head: &str, mut cursor: usize) -> usize {
    while cursor < head.len() {
        let character = head[cursor..]
            .chars()
            .next()
            .expect("cursor should point at a character boundary");
        if character.is_whitespace() || character == ']' {
            break;
        }
        cursor += character.len_utf8();
    }
    cursor
}

fn wikidot_argument_boundary_at(head: &str, mut cursor: usize) -> bool {
    if cursor >= head.len() {
        return true;
    }
    let first = head[cursor..]
        .chars()
        .next()
        .expect("cursor should point at a character boundary");
    if first.is_whitespace() {
        skip_wikidot_argument_whitespace(head, &mut cursor);
        return true;
    }
    wikidot_argument_key_assignment_at(head, cursor)
}

fn wikidot_argument_key_assignment_at(head: &str, mut cursor: usize) -> bool {
    let key_start = cursor;
    while head
        .as_bytes()
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        cursor += 1;
    }
    if cursor == key_start {
        return false;
    }
    let first_key_byte = head.as_bytes()[key_start];
    if !(first_key_byte.is_ascii_alphabetic() || first_key_byte == b'_') {
        return false;
    }
    skip_wikidot_argument_whitespace(head, &mut cursor);
    if head.as_bytes().get(cursor) == Some(&b'!') {
        cursor += 1;
    }
    head.as_bytes().get(cursor) == Some(&b'=')
}

fn skip_wikidot_argument_whitespace(head: &str, cursor: &mut usize) {
    while *cursor < head.len() {
        let character = head[*cursor..]
            .chars()
            .next()
            .expect("cursor should point at a character boundary");
        if !character.is_whitespace() {
            break;
        }
        *cursor += character.len_utf8();
    }
}
