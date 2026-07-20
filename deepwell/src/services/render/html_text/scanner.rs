/*
 * services/render/html_text/scanner.rs
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

//! HTML tokenizer and conservative opaque-element scanning internals.

use super::OPAQUE_ELEMENTS;

#[derive(Clone, Copy, Debug)]
pub(in crate::services::render) enum TagKind {
    Comment,
    BogusComment,
    Cdata,
    Declaration,
    Element { closing: bool },
}

pub(in crate::services::render) fn tag_kind(input: &str) -> Option<TagKind> {
    let bytes = input.as_bytes();
    debug_assert_eq!(bytes.first(), Some(&b'<'));
    match bytes.get(1).copied()? {
        b'!' if input.starts_with("<!--") => Some(TagKind::Comment),
        b'!' if input.starts_with("<![CDATA[") => Some(TagKind::Cdata),
        b'!' | b'?' => Some(TagKind::Declaration),
        b'/' => match bytes.get(2).copied() {
            Some(byte) if byte.is_ascii_alphabetic() => {
                Some(TagKind::Element { closing: true })
            }
            Some(b'>') | None => None,
            Some(_) => Some(TagKind::BogusComment),
        },
        byte if byte.is_ascii_alphabetic() => Some(TagKind::Element { closing: false }),
        _ => None,
    }
}

pub(in crate::services::render) fn protected_construct_end(
    html: &str,
    start: usize,
    kind: TagKind,
) -> Option<usize> {
    match kind {
        TagKind::Comment => comment_end(html, start),
        TagKind::BogusComment => bogus_comment_end(html, start),
        // CDATA sections require foreign-content namespace context, and declarations require their own tokenizer states; this lexical scanner has neither.
        TagKind::Cdata | TagKind::Declaration => None,
        TagKind::Element { .. } => element_tag_end(html, start),
    }
}

fn bogus_comment_end(html: &str, start: usize) -> Option<usize> {
    html[start + 2..]
        .find('>')
        .map(|relative| start + 2 + relative + 1)
}

fn comment_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut cursor = start + "<!--".len();

    if bytes.get(cursor) == Some(&b'>') {
        return Some(cursor + 1);
    }
    if bytes
        .get(cursor..)
        .is_some_and(|rest| rest.starts_with(b"->"))
    {
        return Some(cursor + 2);
    }

    while cursor < bytes.len() {
        if bytes[cursor..].starts_with(b"-->") {
            return Some(cursor + 3);
        }
        if bytes[cursor..].starts_with(b"--!>") {
            return Some(cursor + 4);
        }
        cursor += 1;
    }
    None
}

pub(in crate::services::render) fn opaque_element_end(
    html: &str,
    mut cursor: usize,
    name: &str,
) -> Option<usize> {
    if name == "plaintext" {
        return None;
    }
    if name == "script" {
        return script_element_end(html, cursor);
    }

    if is_raw_text_element(name) {
        return raw_text_element_end(html, cursor, name);
    }

    while let Some(relative) = html[cursor..].find('<') {
        let start = cursor + relative;
        let Some(kind) = tag_kind(&html[start..]) else {
            cursor = start + 1;
            continue;
        };
        let end = protected_construct_end(html, start, kind)?;
        let Some(nested_name) = element_name(&html[start..end]) else {
            cursor = end;
            continue;
        };
        if is_global_tree_builder_barrier(&nested_name)
            || is_table_tree_builder_element(&nested_name)
            || is_foreign_context_barrier(name, &nested_name)
        {
            return None;
        }
        if let TagKind::Element { closing } = kind {
            if closing && nested_name == name {
                return Some(end);
            }
            if !closing {
                if nested_name == name {
                    if is_foreign_self_closing(name, &html[start..end]) {
                        cursor = end;
                        continue;
                    }
                    return None;
                }
                if matches!(nested_name.as_str(), "math" | "svg") {
                    if is_foreign_self_closing(&nested_name, &html[start..end]) {
                        cursor = end;
                        continue;
                    }
                    return None;
                }
                if nested_name == "plaintext" {
                    return None;
                }
                if nested_name == "script" {
                    cursor = script_element_end(html, end)?;
                    continue;
                }
                if is_raw_text_element(&nested_name) {
                    cursor = raw_text_element_end(html, end, &nested_name)?;
                    continue;
                }
            }
        }
        cursor = end;
    }
    None
}

pub(super) fn is_global_tree_builder_barrier(name: &str) -> bool {
    matches!(
        name,
        "applet"
            | "frame"
            | "frameset"
            | "marquee"
            | "noscript"
            | "object"
            | "select"
            | "template"
            | "foreignobject"
            | "annotation-xml"
    )
}

fn is_table_tree_builder_element(name: &str) -> bool {
    matches!(
        name,
        "caption"
            | "col"
            | "colgroup"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
    )
}

fn is_foreign_context_barrier(outer_name: &str, nested_name: &str) -> bool {
    if !matches!(outer_name, "math" | "svg") || nested_name == outer_name {
        return false;
    }

    let integration_point = match outer_name {
        "svg" => matches!(nested_name, "desc" | "foreignobject" | "title"),
        "math" => matches!(
            nested_name,
            "annotation-xml" | "mi" | "mn" | "mo" | "ms" | "mtext"
        ),
        _ => false,
    };
    integration_point
        || OPAQUE_ELEMENTS.contains(&nested_name)
        || matches!(
            nested_name,
            "b" | "big"
                | "blockquote"
                | "body"
                | "br"
                | "center"
                | "dd"
                | "div"
                | "dl"
                | "dt"
                | "em"
                | "embed"
                | "font"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "head"
                | "hr"
                | "i"
                | "img"
                | "li"
                | "listing"
                | "menu"
                | "meta"
                | "nobr"
                | "ol"
                | "p"
                | "ruby"
                | "s"
                | "small"
                | "span"
                | "strike"
                | "strong"
                | "sub"
                | "sup"
                | "tt"
                | "u"
                | "ul"
                | "var"
        )
}

fn is_raw_text_element(name: &str) -> bool {
    matches!(
        name,
        "iframe"
            | "noembed"
            | "noframes"
            | "noscript"
            | "style"
            | "textarea"
            | "title"
            | "xmp"
    )
}

fn raw_text_element_end(html: &str, mut cursor: usize, name: &str) -> Option<usize> {
    while let Some(relative) = html[cursor..].find('<') {
        let start = cursor + relative;
        if matches!(
            tag_kind(&html[start..]),
            Some(TagKind::Element { closing: true })
        ) {
            if !tag_name_at(html, start, true, name) {
                cursor = start + 1;
                continue;
            }
            let end = element_tag_end(html, start)?;
            if element_name(&html[start..end]).as_deref() == Some(name) {
                return Some(end);
            }
        }
        cursor = start + 1;
    }
    None
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptDataState {
    Data,
    Escaped,
    DoubleEscaped,
}

fn script_element_end(html: &str, mut cursor: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut state = ScriptDataState::Data;

    while cursor < bytes.len() {
        match state {
            ScriptDataState::Data => {
                if bytes[cursor..].starts_with(b"<!--") {
                    state = ScriptDataState::Escaped;
                    cursor += 4;
                } else if bytes[cursor] == b'<'
                    && tag_name_at(html, cursor, true, "script")
                {
                    return element_tag_end(html, cursor);
                } else {
                    cursor += 1;
                }
            }
            ScriptDataState::Escaped => {
                if bytes[cursor..].starts_with(b"-->") {
                    state = ScriptDataState::Data;
                    cursor += 3;
                } else if bytes[cursor] == b'<'
                    && tag_name_at(html, cursor, true, "script")
                {
                    return element_tag_end(html, cursor);
                } else if bytes[cursor] == b'<'
                    && tag_name_at(html, cursor, false, "script")
                {
                    state = ScriptDataState::DoubleEscaped;
                    cursor += "<script".len();
                } else {
                    cursor += 1;
                }
            }
            ScriptDataState::DoubleEscaped => {
                if bytes[cursor..].starts_with(b"-->") {
                    state = ScriptDataState::Data;
                    cursor += 3;
                } else if bytes[cursor] == b'<'
                    && tag_name_at(html, cursor, true, "script")
                {
                    state = ScriptDataState::Escaped;
                    cursor += "</script".len();
                } else {
                    cursor += 1;
                }
            }
        }
    }

    None
}

pub(super) fn element_name(tag: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let mut start = 1;
    if bytes.get(start) == Some(&b'/') {
        start += 1;
    }
    let end = bytes[start..]
        .iter()
        .position(|byte| is_html_tag_name_delimiter(*byte))
        .map(|offset| start + offset)?;
    (end > start).then(|| tag[start..end].to_ascii_lowercase())
}

fn tag_name_at(html: &str, start: usize, closing: bool, expected: &str) -> bool {
    let bytes = html.as_bytes();
    let prefix_len = if closing { 2 } else { 1 };
    let name_start = start + prefix_len;
    let name_end = name_start + expected.len();

    bytes.get(start) == Some(&b'<')
        && (!closing || bytes.get(start + 1) == Some(&b'/'))
        && bytes
            .get(name_start..name_end)
            .is_some_and(|name| name.eq_ignore_ascii_case(expected.as_bytes()))
        && bytes
            .get(name_end)
            .is_some_and(|byte| is_html_tag_name_delimiter(*byte))
}

fn is_html_tag_name_delimiter(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\x0C' | b'\r' | b' ' | b'/' | b'>')
}

pub(in crate::services::render) fn is_foreign_self_closing(
    name: &str,
    tag: &str,
) -> bool {
    matches!(name, "math" | "svg") && start_tag_has_self_closing_flag(tag)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TagAttributeState {
    BeforeName,
    Name,
    AfterName,
    BeforeValue,
    DoubleQuotedValue,
    SingleQuotedValue,
    UnquotedValue,
    AfterQuotedValue,
    SelfClosing,
}

fn element_tag_end(html: &str, start: usize) -> Option<usize> {
    scan_element_tag(html, start).map(|(end, _)| end)
}

fn scan_element_tag(html: &str, start: usize) -> Option<(usize, bool)> {
    let bytes = html.as_bytes();
    let name_start = start
        + if bytes.get(start + 1) == Some(&b'/') {
            2
        } else {
            1
        };
    let mut cursor = bytes[name_start..]
        .iter()
        .position(|byte| is_html_tag_name_delimiter(*byte))
        .map(|offset| name_start + offset)?;
    let mut state = TagAttributeState::BeforeName;

    while let Some(&byte) = bytes.get(cursor) {
        let mut reconsume = false;
        state = match state {
            TagAttributeState::BeforeName => match byte {
                byte if is_html_space(byte) => TagAttributeState::BeforeName,
                b'/' => TagAttributeState::SelfClosing,
                b'>' => return Some((cursor + 1, false)),
                _ => TagAttributeState::Name,
            },
            TagAttributeState::Name => match byte {
                byte if is_html_space(byte) => TagAttributeState::AfterName,
                b'/' => TagAttributeState::SelfClosing,
                b'=' => TagAttributeState::BeforeValue,
                b'>' => return Some((cursor + 1, false)),
                _ => TagAttributeState::Name,
            },
            TagAttributeState::AfterName => match byte {
                byte if is_html_space(byte) => TagAttributeState::AfterName,
                b'/' => TagAttributeState::SelfClosing,
                b'=' => TagAttributeState::BeforeValue,
                b'>' => return Some((cursor + 1, false)),
                _ => TagAttributeState::Name,
            },
            TagAttributeState::BeforeValue => match byte {
                byte if is_html_space(byte) => TagAttributeState::BeforeValue,
                b'"' => TagAttributeState::DoubleQuotedValue,
                b'\'' => TagAttributeState::SingleQuotedValue,
                b'>' => return Some((cursor + 1, false)),
                _ => TagAttributeState::UnquotedValue,
            },
            TagAttributeState::DoubleQuotedValue => {
                if byte == b'"' {
                    TagAttributeState::AfterQuotedValue
                } else {
                    TagAttributeState::DoubleQuotedValue
                }
            }
            TagAttributeState::SingleQuotedValue => {
                if byte == b'\'' {
                    TagAttributeState::AfterQuotedValue
                } else {
                    TagAttributeState::SingleQuotedValue
                }
            }
            TagAttributeState::UnquotedValue => match byte {
                byte if is_html_space(byte) => TagAttributeState::BeforeName,
                b'>' => return Some((cursor + 1, false)),
                _ => TagAttributeState::UnquotedValue,
            },
            TagAttributeState::AfterQuotedValue => match byte {
                byte if is_html_space(byte) => TagAttributeState::BeforeName,
                b'/' => TagAttributeState::SelfClosing,
                b'>' => return Some((cursor + 1, false)),
                _ => {
                    reconsume = true;
                    TagAttributeState::BeforeName
                }
            },
            TagAttributeState::SelfClosing => {
                if byte == b'>' {
                    return Some((cursor + 1, true));
                }
                reconsume = true;
                TagAttributeState::BeforeName
            }
        };

        if !reconsume {
            cursor += 1;
        }
    }

    None
}

fn start_tag_has_self_closing_flag(tag: &str) -> bool {
    scan_element_tag(tag, 0)
        .is_some_and(|(end, self_closing)| end == tag.len() && self_closing)
}

fn is_html_space(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\x0C' | b'\r' | b' ')
}
