//! Rendered ListPages fragment normalization and trusted-marker placement.

use super::{CompatHtmlFragments, Range, RenderService, WikidotCompatLinkTitleMap};

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static EMPTY_PARAGRAPH_RESTORE_SCANNED_BYTES: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(super) fn take_empty_paragraph_restore_scanned_bytes() -> usize {
    EMPTY_PARAGRAPH_RESTORE_SCANNED_BYTES.with(|total| total.replace(0))
}

pub(super) fn render_list_pages_form_wiki_value(
    value: &str,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
    compat_html: &mut CompatHtmlFragments,
) -> String {
    let mut rendered = String::with_capacity(value.len());
    for (index, line) in value.split('\n').enumerate() {
        if index > 0 {
            rendered.push_str("<br>\n");
        }
        rendered.push_str(
            &RenderService::render_wikidot_compat_fallback_inline_html_for_page(
                line,
                None,
                link_titles,
            ),
        );
    }
    compat_html.push_html(rendered)
}

pub(super) fn list_pages_rendered_inline_fragment(html: &str) -> String {
    let trimmed = html.trim();
    let Some(inner) = trimmed
        .strip_prefix("<p>")
        .and_then(|value| value.strip_suffix("</p>"))
    else {
        return html.to_owned();
    };
    let (inner, empty_paragraphs) = protect_empty_rendered_paragraphs(inner);
    let joined = inner
        .replace("</p>\n<p>", "\n")
        .replace("</p>\r\n<p>", "\n")
        .replace("</p><p>", "\n");
    restore_empty_paragraphs(joined, &empty_paragraphs)
}

fn restore_empty_paragraphs(joined: String, replacements: &[(String, String)]) -> String {
    if replacements.is_empty() {
        return joined;
    }
    #[cfg(test)]
    EMPTY_PARAGRAPH_RESTORE_SCANNED_BYTES
        .with(|total| total.set(total.get().saturating_add(joined.len())));

    let mut restored = String::with_capacity(joined.len());
    let mut cursor = 0usize;
    for (marker, whitespace) in replacements {
        let Some(relative_start) = joined[cursor..].find(marker) else {
            continue;
        };
        let marker_start = cursor + relative_start;
        restored.push_str(&joined[cursor..marker_start]);
        restored.push_str(whitespace);
        cursor = marker_start + marker.len();
    }
    restored.push_str(&joined[cursor..]);
    restored
}

fn protect_empty_rendered_paragraphs(value: &str) -> (String, Vec<(String, String)>) {
    let mut marker_prefix = '\u{e000}'.to_string();
    while value.contains(&marker_prefix) {
        marker_prefix.push('\u{e001}');
    }
    let mut output = String::with_capacity(value.len());
    let mut replacements = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = value[cursor..].find("<p>") {
        let start = cursor + relative_start;
        let body_start = start + "<p>".len();
        let Some(relative_end) = value[body_start..].find("</p>") else {
            break;
        };
        let end = body_start + relative_end;
        let body = &value[body_start..end];
        if body.trim().is_empty() {
            let marker = format!("{marker_prefix}{}\u{e002}", replacements.len());
            output.push_str(&value[cursor..start]);
            output.push_str(&marker);
            replacements.push((marker, body.to_owned()));
            cursor = end + "</p>".len();
        } else {
            output.push_str(&value[cursor..end + "</p>".len()]);
            cursor = end + "</p>".len();
        }
    }
    output.push_str(&value[cursor..]);
    (output, replacements)
}

pub(super) fn push_list_pages_rendered_fragment(
    html: &str,
    compat_html: &mut CompatHtmlFragments,
) -> String {
    push_list_pages_rendered_fragment_with_mode(html, compat_html, false)
}

pub(super) fn push_list_pages_rendered_fragment_with_mode(
    html: &str,
    compat_html: &mut CompatHtmlFragments,
    force_block: bool,
) -> String {
    // A selected-content render can wrap block-valued HTML in one redundant
    // paragraph. Wikidot's page parser drops that outer paragraph instead of
    // nesting the block paragraphs.
    let html = html
        .strip_prefix("<p><p>")
        .and_then(|value| value.strip_suffix("</p></p>"))
        .map(|value| format!("<p>{value}</p>"))
        .unwrap_or_else(|| html.to_owned());
    // The secondary Ad/AdSense handlers emit this exact empty paragraph as a
    // block marker in an ordinary page render.  Inside a ListPages content
    // value Wikidot leaves the surrounding paragraph boundary in place and
    // does not nest a second empty `<p>`; remove only that handler-owned shape
    // before deciding whether the selected content is inline or block HTML.
    let html = html.replace("<p>\n\n</p>", "\n\n");
    let has_html_block = list_pages_rendered_fragment_has_html_block(&html);
    let rendered = if has_html_block {
        html
    } else {
        list_pages_rendered_inline_fragment(&html)
    };
    // The embed compatibility path intentionally splits one paragraph around
    // Wikidot's terminal "no match" block. Restore that flow fragment as a
    // block so it can close the surrounding row paragraph; other runtime
    // module errors may contain nested trusted markers and must stay inline
    // until their own fragment stack is restored.
    if force_block
        || (list_pages_rendered_fragment_has_block_root(&rendered)
            && rendered.contains(
                r#"<div class="error-block">Sorry, no match for the embedded content.</div>"#,
            ))
        || has_html_block
    {
        compat_html.push_block_html(rendered)
    } else {
        compat_html.push_html(rendered)
    }
}

pub(super) fn list_pages_rendered_fragment_has_html_block(html: &str) -> bool {
    // Wikidot's generated style frames are block-valued even though they do
    // not carry the ordinary HTML-block class.
    html.contains(r#"<iframe "#)
        && (html.contains(r#"class="html-block-iframe""#)
            || html.contains("styleFrame.html"))
}

fn list_pages_rendered_fragment_has_block_root(html: &str) -> bool {
    let trimmed = html.trim_start();
    let Some(paragraph_end) = trimmed.find("</p>") else {
        return false;
    };
    let remainder = trimmed[paragraph_end + "</p>".len()..].trim_start();
    const BLOCK_TAGS: &[&str] = &[
        "address",
        "article",
        "aside",
        "blockquote",
        "div",
        "dl",
        "fieldset",
        "figure",
        "footer",
        "form",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "hr",
        "main",
        "nav",
        "ol",
        "pre",
        "section",
        "table",
        "ul",
    ];
    BLOCK_TAGS.iter().any(|tag| {
        remainder
            .strip_prefix('<')
            .and_then(|value| value.strip_prefix(tag))
            .is_some_and(|value| value.starts_with([' ', '>', '/']))
    })
}

pub(super) fn collect_list_pages_html_body_ranges(source: &str) -> Vec<Range<usize>> {
    const OPEN: &[u8] = b"[[";
    const HEAD_CLOSE: &[u8] = b"]]";
    const CLOSE: &[u8] = b"[[/html]]";

    fn find_ascii_case_insensitive(
        source: &[u8],
        start: usize,
        needle: &[u8],
    ) -> Option<usize> {
        source
            .get(start..)?
            .windows(needle.len())
            .position(|candidate| candidate.eq_ignore_ascii_case(needle))
            .map(|offset| start + offset)
    }

    fn is_complete_block_line(source: &str, start: usize, end: usize) -> bool {
        let line_start = source[..start].rfind('\n').map_or(0, |index| index + 1);
        let line_end = source[end..]
            .find('\n')
            .map_or(source.len(), |offset| end + offset);
        source[line_start..start]
            .chars()
            .all(|character| matches!(character, ' ' | '\t' | '>'))
            && source[end..line_end].chars().all(char::is_whitespace)
    }

    fn is_supported_html_head(head: &str) -> bool {
        if head.eq_ignore_ascii_case("html") {
            return true;
        }
        let Some(variable) = head
            .strip_prefix("%%")
            .and_then(|head| head.strip_suffix("%%html"))
        else {
            return false;
        };
        let variable = variable.to_ascii_lowercase();
        variable == "content"
            || variable
                .strip_prefix("content{")
                .and_then(|value| value.strip_suffix('}'))
                .is_some_and(|section| {
                    !section.is_empty()
                        && section.bytes().all(|byte| byte.is_ascii_digit())
                })
    }

    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    let mut cursor = 0usize;
    while let Some(open) = find_ascii_case_insensitive(bytes, cursor, OPEN) {
        let Some(head_close) =
            find_ascii_case_insensitive(bytes, open + OPEN.len(), HEAD_CLOSE)
        else {
            break;
        };
        let opener_end = head_close + HEAD_CLOSE.len();
        let head = &source[open + OPEN.len()..head_close];
        if !is_supported_html_head(head)
            || !is_complete_block_line(source, open, opener_end)
        {
            cursor = open + OPEN.len();
            continue;
        }
        let body_start = opener_end;
        let mut close_cursor = body_start;
        let close = loop {
            let Some(close) = find_ascii_case_insensitive(bytes, close_cursor, CLOSE)
            else {
                return ranges;
            };
            let close_end = close + CLOSE.len();
            if is_complete_block_line(source, close, close_end) {
                break close;
            }
            close_cursor = close + OPEN.len();
        };
        ranges.push(body_start..close);
        cursor = close + CLOSE.len();
    }
    ranges
}
