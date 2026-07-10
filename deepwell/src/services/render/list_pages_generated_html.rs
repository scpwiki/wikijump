/*
 * services/render/list_pages_generated_html.rs
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

use std::borrow::Cow;
use std::fmt::Write as _;
use uuid::Uuid;

const SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTLISTPAGESHTML";

/// User-controlled ListPages labels hidden from the FTML syntax parser.
///
/// A per-render nonce ties every marker to this side channel. Page source cannot
/// forge a marker that restores a label from another render.
#[derive(Debug, Default)]
pub(super) struct ListPagesGeneratedHtml {
    pub(super) marker_prefix: String,
    labels: Vec<ProtectedLabel>,
}

#[derive(Debug)]
struct ProtectedLabel {
    plain: String,
    html: String,
}

impl ListPagesGeneratedHtml {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn protect_label(&mut self, label: &str) -> String {
        if self.marker_prefix.is_empty() {
            self.marker_prefix =
                format!("{SENTINEL_PREFIX}{}I", Uuid::new_v4().as_simple());
        }
        let index = self.labels.len();
        self.labels.push(ProtectedLabel {
            plain: label.to_owned(),
            html: escape_html_in_any_context(label),
        });
        format!("{}{index}X", self.marker_prefix)
    }
}

pub(super) fn percent_encode_tag_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                write!(&mut encoded, "%{byte:02X}")
                    .expect("writing to a String cannot fail");
            }
        }
    }
    encoded
}

/// Restore indexed markers as HTML-safe text in one forward pass.
pub(super) fn restore_list_pages_generated_html(
    html: String,
    generated_html: &ListPagesGeneratedHtml,
) -> String {
    restore_list_pages_generated_labels(&html, generated_html, true).unwrap_or(html)
}

/// Restore indexed markers as their original text for code-block metadata.
pub(super) fn restore_list_pages_generated_text<'a>(
    text: &'a str,
    generated_html: &ListPagesGeneratedHtml,
) -> Cow<'a, str> {
    restore_list_pages_generated_labels(text, generated_html, false)
        .map_or(Cow::Borrowed(text), Cow::Owned)
}

fn restore_list_pages_generated_labels(
    text: &str,
    generated_html: &ListPagesGeneratedHtml,
    escape_for_html: bool,
) -> Option<String> {
    if generated_html.labels.is_empty() || !text.contains(&generated_html.marker_prefix) {
        return None;
    }

    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(marker_start) = rest.find(&generated_html.marker_prefix) {
        output.push_str(&rest[..marker_start]);
        let marker = &rest[marker_start..];
        let index_start = generated_html.marker_prefix.len();
        let bytes = marker.as_bytes();
        let mut index_end = index_start;
        while bytes.get(index_end).is_some_and(u8::is_ascii_digit) {
            index_end += 1;
        }

        if index_end == index_start || bytes.get(index_end) != Some(&b'X') {
            output.push_str(&marker[..index_start]);
            rest = &marker[index_start..];
            continue;
        }

        let index = marker[index_start..index_end].parse::<usize>().ok();
        let Some(label) = index.and_then(|index| generated_html.labels.get(index)) else {
            output.push_str(&marker[..index_end + 1]);
            rest = &marker[index_end + 1..];
            continue;
        };

        output.push_str(if escape_for_html {
            &label.html
        } else {
            &label.plain
        });
        rest = &marker[index_end + 1..];
    }
    output.push_str(rest);
    Some(output)
}

fn escape_html_in_any_context(value: &str) -> String {
    // Encoding whitespace, quotes, equals, and backticks as well as angle
    // brackets keeps restoration inert even if a marker reached an unquoted
    // attribute or an opaque HTML block instead of ordinary text content.
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
            escaped.push(character);
        } else {
            write!(&mut escaped, "&#x{:X};", character as u32)
                .expect("writing to a String cannot fail");
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_only_labels_from_the_matching_side_channel() {
        let mut generated = ListPagesGeneratedHtml::new();
        let first = generated.protect_label("first <tag>");
        let second = generated.protect_label("second & value");
        let other = ListPagesGeneratedHtml::new().protect_label("other");

        assert_eq!(
            restore_list_pages_generated_html(
                format!("before {first} / {second} / {other} after"),
                &generated,
            ),
            format!(
                "before first&#x20;&#x3C;tag&#x3E; / second&#x20;&#x26;&#x20;value / {other} after"
            ),
        );
        let plain_input = format!("before {first} / {second} after");
        assert_eq!(
            restore_list_pages_generated_text(&plain_input, &generated),
            "before first <tag> / second & value after",
        );
        let malformed = format!("{}oopsX", generated.marker_prefix);
        let unknown = format!("{}9X", generated.marker_prefix);
        let input = format!("{malformed} {unknown}");

        assert_eq!(
            restore_list_pages_generated_html(input.clone(), &generated),
            input,
        );
        assert_eq!(percent_encode_tag_path_segment("safe-._~"), "safe-._~");
        assert_eq!(
            percent_encode_tag_path_segment("] \n日本"),
            "%5D%20%0A%E6%97%A5%E6%9C%AC",
        );
    }

    #[test]
    fn escapes_quotes_for_safe_restoration_even_inside_attributes() {
        let mut generated = ListPagesGeneratedHtml::new();
        let marker = generated.protect_label(r#"owned'" onerror="alert(1)<b>"#);

        assert_eq!(
            restore_list_pages_generated_html(
                format!(r#"<span data-label="{marker}">{marker}</span>"#),
                &generated,
            ),
            concat!(
                r#"<span data-label="owned&#x27;&#x22;&#x20;onerror&#x3D;&#x22;alert&#x28;1&#x29;&#x3C;b&#x3E;">"#,
                "owned&#x27;&#x22;&#x20;onerror&#x3D;&#x22;alert&#x28;1&#x29;&#x3C;b&#x3E;</span>",
            ),
        );
    }
}
