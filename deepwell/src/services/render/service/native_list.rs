//! Native Wikidot list compatibility helpers.

use super::{
    BTreeSet, MAX_NATIVE_LIST_COMPAT_DEPTH, MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS,
    WIKIDOT_EXTERNAL_LINK_REGEX, WIKIDOT_LABELED_LINK_REGEX, WIKIDOT_LOCAL_LINK_REGEX,
    WIKIDOT_QUADRUPLE_LINK_REGEX, WIKIDOT_UNLABELED_LINK_REGEX,
    WIKIDOT_USER_INLINE_REGEX, WIKIDOT_WIKIPEDIA_LINK_REGEX, WikidotCompatLinkTitleMap,
    WikidotWikipediaLink, build_wikidot_wikipedia_link, escape_list_pages_html_attr,
    escape_list_pages_html_text, native_list_page_link_ref,
    sanitize_wikidot_compat_inline_tag,
};

pub(in crate::services::render) fn render_native_bullet_list_with_wikipedia_links(
    lines: &[&str],
    wikipedia_links: &mut Vec<WikidotWikipediaLink>,
) -> String {
    let items: Vec<_> = lines
        .iter()
        .filter_map(|line| native_bullet_list_item(line))
        .collect();
    let base_depth = items.iter().map(|(depth, _)| *depth).min().unwrap_or(0);
    let mut output = String::new();
    let mut current_depth = 0usize;
    let mut open_li = false;

    output.push_str(r#"<ul data-wikijump-compat-list="1">"#);
    output.push('\n');

    for (index, &(raw_depth, content)) in items.iter().enumerate() {
        let depth = raw_depth
            .saturating_sub(base_depth)
            .min(MAX_NATIVE_LIST_COMPAT_DEPTH);
        let has_children = items.get(index + 1).is_some_and(|(next_depth, _)| {
            next_depth
                .saturating_sub(base_depth)
                .min(MAX_NATIVE_LIST_COMPAT_DEPTH)
                > depth
        });

        if depth > current_depth {
            while current_depth < depth {
                output.push_str("<ul>\n");
                current_depth += 1;
            }
        } else if depth < current_depth {
            if open_li {
                output.push_str("</li>\n");
            }

            while current_depth > depth {
                output.push_str("</ul>\n</li>\n");
                current_depth -= 1;
            }
        } else if open_li {
            output.push_str("</li>\n");
        }

        output.push_str("<li>");
        output.push_str(&render_native_list_item_content(
            content,
            has_children,
            wikipedia_links,
        ));
        open_li = true;
    }

    if open_li {
        output.push_str("</li>\n");
    }

    while current_depth > 0 {
        output.push_str("</ul>\n</li>\n");
        current_depth -= 1;
    }

    output.push_str("</ul>\n");
    output
}

fn render_native_list_item_content(
    content: &str,
    has_children: bool,
    wikipedia_links: &mut Vec<WikidotWikipediaLink>,
) -> String {
    let rendered =
        render_native_list_inline_html_with_wikipedia_links(content, wikipedia_links);
    if has_children && !rendered.contains("<a ") {
        format!(
            r#"<a href="javascript:;">{rendered}
</a>"#
        )
    } else {
        rendered
    }
}

#[cfg(test)]
pub(in crate::services::render) fn find_balanced_ul_end(html: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut cursor = 0usize;

    while let Some(offset) = html[cursor..].find('<') {
        cursor += offset;

        if html[cursor..].starts_with("<ul") {
            depth += 1;
            cursor += "<ul".len();
            continue;
        }

        if html[cursor..].starts_with("</ul>") {
            if depth == 0 {
                return None;
            }

            depth -= 1;
            cursor += "</ul>".len();
            if depth == 0 {
                return Some(cursor);
            }
            continue;
        }

        cursor += '<'.len_utf8();
    }

    None
}

pub(in crate::services::render) fn native_bullet_list_item(
    line: &str,
) -> Option<(usize, &str)> {
    let trimmed_end = line.trim_end_matches(['\r', '\n']);
    let depth = trimmed_end
        .as_bytes()
        .iter()
        .take_while(|&&byte| byte == b' ')
        .count();
    trimmed_end[depth..]
        .strip_prefix("* ")
        .map(|content| (depth, content))
}

pub(in crate::services::render) fn native_numbered_list_content(
    line: &str,
) -> Option<&str> {
    let trimmed = line.trim_start_matches(' ');
    trimmed
        .strip_prefix("# ")
        .map(|content| content.trim_end_matches(['\r', '\n']))
}

pub(in crate::services::render) fn render_list_pages_numbered_rows_with_titles(
    value: &str,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
) -> String {
    let lines = value.split_inclusive('\n').collect::<Vec<_>>();
    let mut output = String::with_capacity(value.len());
    let mut index = 0;

    while index < lines.len() {
        if native_numbered_list_content(lines[index]).is_some() {
            let list_start = index;
            let mut list_end = index;
            while list_end < lines.len()
                && native_numbered_list_content(lines[list_end]).is_some()
            {
                list_end += 1;
                while list_end < lines.len()
                    && native_numbered_list_span_continuation(lines[list_end]).is_some()
                {
                    list_end += 1;
                }
            }
            if lines[list_start..list_end]
                .iter()
                .any(|line| line.to_ascii_lowercase().contains("[[footnote"))
            {
                // FTML owns footnote allocation and the corresponding footer
                // registry. Leave the complete ordered-list run as syntax
                // when any item contains a footnote instead of partially
                // rendering the item through the compatibility inline helper.
                for line in &lines[list_start..list_end] {
                    output.push_str(line);
                }
                index = list_end;
                continue;
            }
            output.push_str("<ol data-wikijump-compat-listpages=\"1\">\n");
            while index < lines.len() {
                let Some(content) = native_numbered_list_content(lines[index]) else {
                    break;
                };
                let has_span_continuation = lines
                    .get(index + 1)
                    .and_then(|line| native_numbered_list_span_continuation(line))
                    .is_some();
                let content = if has_span_continuation {
                    content.strip_suffix(" _").unwrap_or(content)
                } else {
                    content
                };
                output.push_str("<li>");
                output.push_str(&render_native_list_inline_html_with_titles(
                    content,
                    link_titles,
                ));
                index += 1;
                while index < lines.len()
                    && let Some((continuation, continuation_body)) =
                        native_numbered_list_span_continuation(lines[index])
                {
                    output.push_str("<br>\n");
                    if !continuation_body
                        .trim_matches([' ', '\t', '\r', '\n'])
                        .is_empty()
                    {
                        output.push_str(&render_native_list_inline_html_with_titles(
                            continuation,
                            link_titles,
                        ));
                    }
                    index += 1;
                }
                output.push_str("</li>\n");
            }
            output.push_str("</ol>\n");
        } else {
            output.push_str(lines[index]);
            index += 1;
        }
    }

    output
}

fn native_numbered_list_span_continuation(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_end_matches(['\r', '\n']);
    let trimmed = line.trim();
    let marker_end = trimmed.find("]]")? + "]]".len();
    wikidot_inline_span_marker_open(&trimmed[..marker_end])?;
    let body = &trimmed[marker_end..];
    let close_start = find_matching_wikidot_span_close(body)?;
    (close_start + "[[/span]]".len() == body.len())
        .then_some((trimmed, &body[..close_start]))
}

pub(in crate::services::render) fn render_list_pages_table_rows(
    value: &str,
) -> Option<String> {
    if !list_pages_body_has_table_rows(value) {
        return None;
    }

    let mut rows = Vec::new();
    for line in value.lines().filter(|line| !line.trim().is_empty()) {
        let trimmed = line.trim();
        let center = trimmed.starts_with("||=");
        let header = trimmed.starts_with("||~");
        let cell = trimmed
            .trim_start_matches("||=")
            .trim_start_matches("||~")
            .trim_end_matches("||")
            .trim();
        rows.push((header, center, render_list_pages_table_inline_html(cell)));
    }

    if rows.is_empty() {
        return None;
    }

    let mut output = String::from(
        "<table class=\"wiki-content-table\" data-wikijump-compat-listpages=\"1\">",
    );
    for (header, center, cell) in rows {
        output.push_str("<tr>");
        let tag = if header { "th" } else { "td" };
        output.push('<');
        output.push_str(tag);
        if center {
            output.push_str(" style=\"text-align: center;\"");
        }
        output.push('>');
        output.push_str(&cell);
        output.push_str("</");
        output.push_str(tag);
        output.push_str("></tr>");
    }
    output.push_str("</table>");
    Some(output)
}

fn list_pages_body_has_table_rows(value: &str) -> bool {
    let mut any = false;
    for line in value.lines().filter(|line| !line.trim().is_empty()) {
        any = true;
        let trimmed = line.trim();
        if !trimmed.starts_with("||") || !trimmed.ends_with("||") {
            return false;
        }
        if !trimmed.starts_with("||=") && !trimmed.starts_with("||~") {
            return false;
        }
    }
    any
}

pub(super) fn render_list_pages_table_inline_html(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut strong = false;
    for segment in value.split("**") {
        if strong {
            output.push_str("<strong>");
        }
        push_list_pages_table_inline_segment(&mut output, segment);
        if strong {
            output.push_str("</strong>");
        }
        strong = !strong;
    }
    output
}

fn push_list_pages_table_inline_segment(output: &mut String, value: &str) {
    let mut rest = value;
    while let Some(start) = rest.find('<') {
        let (before, after_start) = rest.split_at(start);
        output.push_str(&escape_list_pages_html_text(before));
        if let Some(end) = after_start.find('>') {
            let (tag, after_tag) = after_start.split_at(end + 1);
            if let Some(tag) = sanitize_wikidot_compat_inline_tag(tag) {
                output.push_str(&tag);
            } else {
                output.push_str(&escape_list_pages_html_text(tag));
            }
            rest = after_tag;
        } else {
            output.push_str(&escape_list_pages_html_text(after_start));
            return;
        }
    }
    output.push_str(&escape_list_pages_html_text(rest));
}

pub(in crate::services::render) fn render_native_list_inline_html(value: &str) -> String {
    render_native_list_inline_html_with_titles(value, None)
}

fn render_native_list_inline_html_with_wikipedia_links(
    value: &str,
    wikipedia_links: &mut Vec<WikidotWikipediaLink>,
) -> String {
    render_native_list_inline_html_with_titles_and_wikipedia_links(
        value,
        None,
        Some(wikipedia_links),
    )
}

pub(in crate::services::render) fn render_native_list_inline_html_with_titles(
    value: &str,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
) -> String {
    render_native_list_inline_html_with_titles_and_wikipedia_links(
        value,
        link_titles,
        None,
    )
}

fn render_native_list_inline_html_with_titles_and_wikipedia_links(
    value: &str,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
    mut wikipedia_links: Option<&mut Vec<WikidotWikipediaLink>>,
) -> String {
    let escaped = render_native_list_inline_wikidot_spans(value);
    let with_quadruple_links = WIKIDOT_QUADRUPLE_LINK_REGEX
        .replace_all(&escaped, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(&captures["target"], None, link_titles)
        })
        .into_owned();
    let with_labeled_links = WIKIDOT_LABELED_LINK_REGEX
        .replace_all(&with_quadruple_links, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(
                &captures["target"],
                Some(&captures["label"]),
                link_titles,
            )
        })
        .into_owned();
    let with_unlabeled_links = WIKIDOT_UNLABELED_LINK_REGEX
        .replace_all(&with_labeled_links, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(&captures["target"], None, link_titles)
        })
        .into_owned();
    let with_local_links = WIKIDOT_LOCAL_LINK_REGEX
        .replace_all(&with_unlabeled_links, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(
                &captures["target"],
                Some(&captures["label"]),
                link_titles,
            )
        })
        .into_owned();
    let with_user_links = WIKIDOT_USER_INLINE_REGEX
        .replace_all(&with_local_links, |captures: &regex::Captures<'_>| {
            render_native_list_wikidot_user(&captures["name"])
        })
        .into_owned();
    let with_wikipedia_links = WIKIDOT_WIKIPEDIA_LINK_REGEX
        .replace_all(&with_user_links, |captures: &regex::Captures<'_>| {
            let link = build_wikidot_wikipedia_link(
                &captures["target"],
                captures.name("label").map(|matched| matched.as_str()),
            );
            let anchor = link.anchor.clone();
            if let Some(links) = wikipedia_links.as_deref_mut() {
                links.push(link);
            }
            anchor
        })
        .into_owned();

    let with_external_links = WIKIDOT_EXTERNAL_LINK_REGEX
        .replace_all(&with_wikipedia_links, |captures: &regex::Captures<'_>| {
            format!(
                r#"<a href="{url}">{label}</a>"#,
                url = escape_list_pages_html_attr(&captures["url"]),
                label = captures["label"].to_owned(),
            )
        })
        .into_owned();

    render_native_list_inline_wikidot_italics(&with_external_links)
}

fn render_native_list_inline_wikidot_italics(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(tag_start) = rest.find('<') {
        let (before, after_start) = rest.split_at(tag_start);
        output.push_str(&render_native_list_text_italics(before));

        let Some(tag_end) = after_start.find('>') else {
            output.push_str(&render_native_list_text_italics(after_start));
            return output;
        };
        let (tag, after_tag) = after_start.split_at(tag_end + 1);
        output.push_str(tag);
        rest = after_tag;
    }

    output.push_str(&render_native_list_text_italics(rest));
    output
}

fn render_native_list_text_italics(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(open) = find_wikidot_italic_open(rest) {
        output.push_str(&rest[..open]);
        let after_open = &rest[open + "//".len()..];
        let Some(close) = find_wikidot_italic_close(after_open) else {
            output.push_str(&rest[open..]);
            return output;
        };

        output.push_str("<em>");
        output.push_str(&after_open[..close]);
        output.push_str("</em>");
        rest = &after_open[close + "//".len()..];
    }

    output.push_str(rest);
    output
}

pub(in crate::services::render) fn render_native_list_inline_wikidot_strong(
    value: &str,
) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(tag_start) = rest.find('<') {
        let (before, after_start) = rest.split_at(tag_start);
        output.push_str(&render_native_list_text_strong(before));

        let Some(tag_end) = after_start.find('>') else {
            output.push_str(&render_native_list_text_strong(after_start));
            return output;
        };
        let (tag, after_tag) = after_start.split_at(tag_end + 1);
        output.push_str(tag);
        rest = after_tag;
    }

    output.push_str(&render_native_list_text_strong(rest));
    output
}

fn render_native_list_text_strong(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(open) = rest.find("**") {
        output.push_str(&rest[..open]);
        let after_open = &rest[open + "**".len()..];
        let Some(close) = after_open.find("**") else {
            output.push_str(&rest[open..]);
            return output;
        };

        output.push_str("<strong>");
        output.push_str(&after_open[..close]);
        output.push_str("</strong>");
        rest = &after_open[close + "**".len()..];
    }

    output.push_str(rest);
    output
}

pub(in crate::services::render) fn render_native_list_inline_wikidot_underlines(
    value: &str,
) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(tag_start) = rest.find('<') {
        let (before, after_start) = rest.split_at(tag_start);
        output.push_str(&render_native_list_text_underlines(before));

        let Some(tag_end) = after_start.find('>') else {
            output.push_str(&render_native_list_text_underlines(after_start));
            return output;
        };
        let (tag, after_tag) = after_start.split_at(tag_end + 1);
        output.push_str(tag);
        rest = after_tag;
    }

    output.push_str(&render_native_list_text_underlines(rest));
    output
}

fn render_native_list_text_underlines(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(open) = rest.find("__") {
        output.push_str(&rest[..open]);
        let after_open = &rest[open + "__".len()..];
        let Some(close) = after_open.find("__") else {
            output.push_str(&rest[open..]);
            return output;
        };

        output.push_str("<u>");
        output.push_str(&after_open[..close]);
        output.push_str("</u>");
        rest = &after_open[close + "__".len()..];
    }

    output.push_str(rest);
    output
}

fn find_wikidot_italic_open(value: &str) -> Option<usize> {
    let mut cursor = 0usize;
    while let Some(offset) = value[cursor..].find("//") {
        let marker = cursor + offset;
        let previous = value[..marker].chars().next_back();
        let next = value[marker + "//".len()..].chars().next();
        if previous == Some(':')
            || next.is_none_or(|character| character.is_whitespace() || character == '/')
        {
            cursor = marker + "//".len();
            continue;
        }
        return Some(marker);
    }
    None
}

fn find_wikidot_italic_close(value: &str) -> Option<usize> {
    let mut cursor = 0usize;
    while let Some(offset) = value[cursor..].find("//") {
        let marker = cursor + offset;
        let previous = value[..marker].chars().next_back();
        let next = value[marker + "//".len()..].chars().next();
        if previous.is_none_or(char::is_whitespace) || next == Some('/') {
            cursor = marker + "//".len();
            continue;
        }
        return Some(marker);
    }
    None
}

pub(in crate::services::render) fn render_native_list_inline_wikidot_spans(
    value: &str,
) -> String {
    render_native_list_inline_wikidot_spans_at_depth(value, 0)
}

pub(super) const MAX_NATIVE_LIST_WIKIDOT_SPAN_NESTING: usize = 64;

fn render_native_list_inline_wikidot_spans_at_depth(value: &str, depth: usize) -> String {
    if depth >= MAX_NATIVE_LIST_WIKIDOT_SPAN_NESTING {
        return escape_list_pages_html_text(value);
    }

    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(start) = rest.find("[[span") {
        let (before, marker_start) = rest.split_at(start);
        output.push_str(&escape_list_pages_html_text(before));

        let Some(marker_end) = marker_start.find("]]") else {
            output.push_str(&escape_list_pages_html_text(marker_start));
            return output;
        };
        let marker = &marker_start[..marker_end + 2];
        let after_marker = &marker_start[marker_end + 2..];

        let Some(open_tag) = wikidot_inline_span_marker_open(marker) else {
            output.push_str(&escape_list_pages_html_text(marker));
            rest = after_marker;
            continue;
        };

        let Some(close_start) = find_matching_wikidot_span_close(after_marker) else {
            output.push_str(&escape_list_pages_html_text(marker_start));
            return output;
        };

        output.push_str(&open_tag);
        output.push_str(&render_native_list_inline_wikidot_spans_at_depth(
            &after_marker[..close_start],
            depth + 1,
        ));
        output.push_str("</span>");
        rest = &after_marker[close_start + "[[/span]]".len()..];
    }

    output.push_str(&escape_list_pages_html_text(rest));
    output
}

fn find_matching_wikidot_span_close(value: &str) -> Option<usize> {
    const SPAN_CLOSE: &[u8] = b"[[/span]]";
    const SPAN_OPEN_PREFIX: &[u8] = b"[[span";
    const SPAN_MARKER_END: &[u8] = b"]]";

    let bytes = value.as_bytes();
    let mut depth = 1_usize;
    let mut offset = 0;

    while offset < bytes.len() {
        if bytes[offset..].starts_with(SPAN_CLOSE) {
            if depth == 1 {
                return Some(offset);
            }
            depth -= 1;
            offset += SPAN_CLOSE.len();
            continue;
        }

        if bytes[offset..].starts_with(SPAN_OPEN_PREFIX) {
            let open = offset;
            offset += SPAN_OPEN_PREFIX.len();

            loop {
                if bytes[offset..].starts_with(SPAN_CLOSE) {
                    // The candidate opener is malformed because a real
                    // closer appears before its terminating marker. Leave the
                    // cursor on that closer so the outer span can consume it.
                    break;
                }
                if bytes[offset..].starts_with(SPAN_MARKER_END) {
                    let marker_end = offset + SPAN_MARKER_END.len();
                    let marker = &value[open..marker_end];
                    if wikidot_inline_span_marker_open(marker).is_some() {
                        depth += 1;
                    }
                    offset = marker_end;
                    break;
                }
                offset += 1;
                if offset >= bytes.len() {
                    return None;
                }
            }
            continue;
        }

        offset += 1;
    }

    None
}

pub(in crate::services::render) fn wikidot_inline_span_marker_open(
    marker: &str,
) -> Option<String> {
    let marker = marker.trim();
    if !marker.ends_with("]]") {
        return None;
    }

    let inner = marker.strip_prefix("[[")?.strip_suffix("]]")?.trim();
    if inner.len() < "span".len() || !inner[.."span".len()].eq_ignore_ascii_case("span") {
        return None;
    }
    if inner.len() > "span".len()
        && !inner
            .as_bytes()
            .get("span".len())
            .is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }
    if inner.contains(['<', '>']) {
        return None;
    }

    sanitize_wikidot_compat_inline_tag(&format!("<{inner}>"))
}

pub(super) fn render_native_list_page_link(
    target: &str,
    label: Option<&str>,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
) -> String {
    let target = target.trim();
    let external_target = target
        .strip_prefix('*')
        .filter(|target| target.starts_with("http://") || target.starts_with("https://"));
    let page_ref = native_list_page_link_ref(target);
    let page_is_missing = page_ref.as_ref().is_some_and(|page_ref| {
        link_titles.is_some_and(|titles| titles.page_is_missing(page_ref))
    });
    let explicitly_empty_label = label.is_some_and(|label| label.trim().is_empty());
    let label = label
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            if explicitly_empty_label && page_is_missing {
                escape_list_pages_html_text(target)
            } else if let Some(external_target) = external_target {
                escape_list_pages_html_text(external_target)
            } else {
                native_list_page_link_title_label(target, link_titles)
                    .unwrap_or_else(|| native_list_page_link_default_label(target))
            }
        });
    if let Some(external_target) = external_target {
        return format!(
            r#"<a target="_blank" href="{href}">{label}</a>"#,
            href = escape_list_pages_html_attr(external_target),
            label = label,
        );
    }
    let href = native_list_page_link_href(target);
    let class = if page_is_missing {
        r#" class="newpage""#
    } else {
        ""
    };
    format!(
        r#"<a{class} href="{href}">{label}</a>"#,
        class = class,
        href = escape_list_pages_html_attr(&href),
        label = label,
    )
}

fn native_list_page_link_title_label(
    target: &str,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
) -> Option<String> {
    let slug = native_list_page_link_slug(target)?;
    link_titles?.title(&slug).map(escape_list_pages_html_text)
}

fn native_list_page_link_href(target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        return target.to_owned();
    }

    let mut slug = String::with_capacity(target.len());
    let mut previous_dash = false;
    for character in target.trim().chars() {
        if character.is_whitespace() || character == '_' {
            if !previous_dash {
                slug.push('-');
                previous_dash = true;
            }
        } else {
            for lowercase in character.to_lowercase() {
                slug.push(lowercase);
            }
            previous_dash = character == '-';
        }
    }

    format!("/{}", slug.trim_matches('-'))
}

pub(in crate::services::render) fn native_list_page_link_slug(
    target: &str,
) -> Option<String> {
    let target = target.trim();
    if target.is_empty()
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with('#')
        || target.starts_with(':')
        || target.contains(['?', '&', '=', '#', '<', '>', '"', '\''])
    {
        return None;
    }

    let href = native_list_page_link_href(target);
    let slug = href.strip_prefix('/')?.trim_matches('-');
    if slug.is_empty()
        || slug.len() > 256
        || slug.contains('/')
        || !slug.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':')
        })
    {
        return None;
    }

    Some(slug.to_owned())
}

pub(in crate::services::render) fn collect_wikidot_compat_empty_label_link_slugs(
    wikitext: &str,
) -> BTreeSet<String> {
    let mut slugs = BTreeSet::new();
    for captures in WIKIDOT_QUADRUPLE_LINK_REGEX.captures_iter(wikitext) {
        if let Some(slug) = native_list_page_link_slug(&captures["target"]) {
            slugs.insert(slug);
        }
        if slugs.len() >= MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS {
            return slugs;
        }
    }
    for captures in WIKIDOT_UNLABELED_LINK_REGEX.captures_iter(wikitext) {
        if let Some(slug) = native_list_page_link_slug(&captures["target"]) {
            slugs.insert(slug);
        }
        if slugs.len() >= MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS {
            return slugs;
        }
    }
    for captures in WIKIDOT_LABELED_LINK_REGEX.captures_iter(wikitext) {
        if !captures["label"].trim().is_empty() {
            continue;
        }
        if let Some(slug) = native_list_page_link_slug(&captures["target"]) {
            slugs.insert(slug);
        }
        if slugs.len() >= MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS {
            return slugs;
        }
    }

    slugs
}

pub(super) fn native_list_page_link_default_label(target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        return target.to_owned();
    }
    if target.contains(char::is_whitespace) {
        return target.to_owned();
    }
    if let Some(label) = native_list_scp_style_page_link_default_label(target) {
        return label;
    }

    target
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = String::new();
                    for uppercase in first.to_uppercase() {
                        word.push(uppercase);
                    }
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn native_list_scp_style_page_link_default_label(target: &str) -> Option<String> {
    let mut parts = target.trim().split('-');
    let prefix = parts.next()?;
    if !prefix.eq_ignore_ascii_case("scp") {
        return None;
    }

    let number = parts.next()?;
    if number.is_empty() || !number.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    let mut label = format!("SCP-{number}");
    for part in parts {
        if part.is_empty()
            || !part
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
        {
            return None;
        }
        label.push('-');
        label.push_str(&part.to_ascii_uppercase());
    }

    Some(label)
}

fn render_native_list_wikidot_user(name: &str) -> String {
    let name = name.trim();
    format!(
        concat!(
            r#"<span class="printuser">"#,
            r#"<a href="http://www.wikidot.com/user:info/{slug}">{name}</a>"#,
            r#"</span>"#
        ),
        slug = escape_list_pages_html_attr(name),
        name = escape_list_pages_html_text(name),
    )
}
