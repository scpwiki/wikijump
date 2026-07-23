/*
 * services/render/wikidot_residual_markers.rs
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

use super::html_text::html_data_segments;
use super::service::*;
use std::ops::Range;

impl RenderService {
    pub(super) fn restore_residual_wikidot_div_paragraph_markers(html: &str) -> String {
        let mut restored_open_count = 0usize;

        let restored = WIKIDOT_RESIDUAL_DIV_PARAGRAPH_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                if let Some(marker) = captures.name("open") {
                    let marker = marker
                        .as_str()
                        .replace("&quot;", "\"")
                        .replace("&#34;", "\"");
                    if let Some(attributes) =
                        Self::wikidot_residual_div_attributes(&marker)
                    {
                        restored_open_count += 1;
                        return format!("<div{attributes}>");
                    }

                    return captures.get(0).unwrap().as_str().to_owned();
                }

                if restored_open_count == 0 {
                    return captures.get(0).unwrap().as_str().to_owned();
                }

                restored_open_count -= 1;
                "</div>".to_owned()
            })
            .into_owned();

        Self::restore_standalone_residual_wikidot_div_markers(&restored)
    }

    pub(super) fn restore_residual_wikidot_span_markers(html: &str) -> String {
        let mut open_markers: Vec<(Range<usize>, String)> = Vec::new();
        let mut replacements: Vec<(Range<usize>, String)> = Vec::new();

        for segment in html_data_segments(html) {
            if !segment.continues_from_previous {
                open_markers.clear();
            }
            let data_range = segment.range;
            let data = &html[data_range.clone()];
            let mut cursor = 0;
            while cursor < data.len() {
                let open = data[cursor..].find("[[span");
                let close = data[cursor..].find("[[/span]]");
                let (offset, closing) = match (open, close) {
                    (Some(open), Some(close)) if close < open => (close, true),
                    (Some(open), _) => (open, false),
                    (None, Some(close)) => (close, true),
                    (None, None) => break,
                };
                let start = cursor + offset;

                if closing {
                    let end = start + "[[/span]]".len();
                    if let Some((open_range, open_tag)) = open_markers.pop() {
                        replacements.push((open_range, open_tag));
                        replacements.push((
                            data_range.start + start..data_range.start + end,
                            "</span>".to_owned(),
                        ));
                    }
                    cursor = end;
                    continue;
                }

                let marker_start = &data[start..];
                let Some(relative_end) = marker_start.find("]]") else {
                    break;
                };
                let end = start + relative_end + 2;
                let marker = &data[start..end];
                let decoded_marker = Self::decode_residual_wikidot_marker_quotes(marker);
                if let Some(open_tag) = wikidot_inline_span_marker_open(&decoded_marker) {
                    open_markers.push((
                        data_range.start + start..data_range.start + end,
                        open_tag,
                    ));
                }
                cursor = end;
            }
        }

        if replacements.is_empty() {
            return html.to_owned();
        }
        replacements.sort_by_key(|(range, _)| range.start);

        let mut output = String::with_capacity(html.len());
        let mut cursor = 0;
        for (range, replacement) in replacements {
            output.push_str(&html[cursor..range.start]);
            output.push_str(&replacement);
            cursor = range.end;
        }
        output.push_str(&html[cursor..]);
        output
    }

    pub(super) fn restore_residual_wikidot_alignment_markers(html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let mut alignment_stack: Vec<&'static str> = Vec::new();
        let mut raw_text_depth = 0usize;

        for line in html.split_inclusive('\n') {
            let (line_body, line_end) = line
                .strip_suffix('\n')
                .map_or((line, ""), |body| (body, "\n"));
            let trimmed = line_body.trim();
            let protected = raw_text_depth > 0;

            if !protected {
                if let Some((alignment, replacement)) =
                    Self::residual_wikidot_alignment_open_replacement(trimmed)
                {
                    alignment_stack.push(alignment);
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        replacement,
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }

                if let Some(alignment) = Self::residual_wikidot_alignment_close(trimmed)
                    && alignment_stack.last().copied() == Some(alignment)
                {
                    alignment_stack.pop();
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        "</div>",
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }
            }

            output.push_str(line_body);
            output.push_str(line_end);
            raw_text_depth =
                Self::update_residual_div_raw_text_depth(raw_text_depth, line_body);
        }

        Self::restore_residual_wikidot_alignment_html_markers(&output)
    }

    pub(super) fn restore_residual_wikidot_alignment_html_markers(html: &str) -> String {
        const MARKERS: &[(&str, &str, &str, bool)] = &[
            (
                "<p>[[=]]</p>",
                "center",
                r#"<div style="text-align: center;">"#,
                false,
            ),
            (
                "<p>[[<]]</p>",
                "left",
                r#"<div style="text-align: left;">"#,
                false,
            ),
            (
                "<p>[[&lt;]]</p>",
                "left",
                r#"<div style="text-align: left;">"#,
                false,
            ),
            (
                "<p>[[>]]</p>",
                "right",
                r#"<div style="text-align: right;">"#,
                false,
            ),
            (
                "<p>[[&gt;]]</p>",
                "right",
                r#"<div style="text-align: right;">"#,
                false,
            ),
            ("<p>[[/=]]</p>", "center", "</div>", true),
            ("<br>[[/=]]<br>", "center", "</div><br>", true),
            ("<br/>[[/=]]<br/>", "center", "</div><br/>", true),
            ("<br />[[/=]]<br />", "center", "</div><br />", true),
            ("<p>[[/<]]</p>", "left", "</div>", true),
            ("<p>[[/&lt;]]</p>", "left", "</div>", true),
            ("<br>[[/<]]<br>", "left", "</div><br>", true),
            ("<br>[[/&lt;]]<br>", "left", "</div><br>", true),
            ("<p>[[/>]]</p>", "right", "</div>", true),
            ("<p>[[/&gt;]]</p>", "right", "</div>", true),
            ("<br>[[/>]]<br>", "right", "</div><br>", true),
            ("<br>[[/&gt;]]<br>", "right", "</div><br>", true),
        ];

        let mut output = String::with_capacity(html.len());
        let mut rest = html;
        let mut alignment_stack: Vec<&'static str> = Vec::new();

        while let Some(position) = rest.find('<') {
            output.push_str(&rest[..position]);
            rest = &rest[position..];

            let Some((marker, alignment, replacement, is_close)) = MARKERS
                .iter()
                .find(|(marker, ..)| rest.starts_with(marker))
                .map(|(marker, alignment, replacement, is_close)| {
                    (*marker, *alignment, *replacement, *is_close)
                })
            else {
                output.push('<');
                rest = &rest['<'.len_utf8()..];
                continue;
            };

            if is_close {
                if alignment_stack.last().copied() == Some(alignment) {
                    alignment_stack.pop();
                    output.push_str(replacement);
                } else {
                    output.push_str(marker);
                }
            } else {
                alignment_stack.push(alignment);
                output.push_str(replacement);
            }
            rest = &rest[marker.len()..];
        }

        output.push_str(rest);
        output
    }

    pub(super) fn restore_residual_wikidot_separator_markers(html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let mut raw_text_depth = 0usize;

        for line in html.split_inclusive('\n') {
            let (line_body, line_end) = line
                .strip_suffix('\n')
                .map_or((line, ""), |body| (body, "\n"));
            let trimmed = line_body.trim();
            let protected = raw_text_depth > 0;

            if !protected {
                if Self::residual_wikidot_horizontal_rule_line(trimmed) {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        "<hr>",
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }

                if trimmed == "@@ @@" {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        r#"<p><span style="white-space: pre-wrap;"> </span></p>"#,
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }

                if trimmed == "~~~~" {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        r#"<div style="clear:both; height: 0px; font-size: 1px"></div>"#,
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }
            }

            output.push_str(line_body);
            output.push_str(line_end);
            raw_text_depth =
                Self::update_residual_div_raw_text_depth(raw_text_depth, line_body);
        }

        output
    }

    pub(super) fn restore_residual_wikidot_heading_markers(html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let mut raw_text_depth = 0usize;

        for line in html.split_inclusive('\n') {
            let (line_body, line_end) = line
                .strip_suffix('\n')
                .map_or((line, ""), |body| (body, "\n"));
            let trimmed = line_body.trim();
            let protected = raw_text_depth > 0;

            if !protected {
                if Self::residual_wikidot_content_section_line(trimmed) {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        "",
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }

                if let Some((level, body)) =
                    Self::residual_wikidot_heading_replacement(trimmed)
                {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        &format!("<h{level}><span>{body}</span></h{level}>"),
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }
            }

            output.push_str(line_body);
            output.push_str(line_end);
            raw_text_depth =
                Self::update_residual_div_raw_text_depth(raw_text_depth, line_body);
        }

        output
    }
}
