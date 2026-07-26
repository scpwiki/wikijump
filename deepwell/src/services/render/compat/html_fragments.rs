//! Compatibility-local provenance for HTML produced by trusted runtime producers.

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use uuid::Uuid;

use super::super::html_text::{
    HtmlDataSegment, OPAQUE_ELEMENTS, TagKind, html_data_segments,
    is_foreign_self_closing, opaque_element_end, protected_construct_end, tag_kind,
};
use super::super::literal_regions::LiteralRegionIndex;

pub(in crate::services::render) const COMPAT_HTML_MARKER_PREFIX: &str =
    "WIKIJUMPWIKIDOTCOMPATHTML";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(in crate::services::render) struct CompatHtmlFragments {
    namespace: String,
    fragments: Vec<CompatFragment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CompatFragment {
    Html(String),
    BlockHtml(String),
    Plain { plain: String, html: String },
}

impl CompatHtmlFragments {
    pub(in crate::services::render) fn new(untrusted_source: &str) -> Self {
        let namespace = loop {
            let candidate =
                format!("{COMPAT_HTML_MARKER_PREFIX}{}I", Uuid::new_v4().as_simple(),);
            if !untrusted_source.contains(&candidate) {
                break candidate;
            }
        };
        Self {
            namespace,
            fragments: Vec::new(),
        }
    }

    pub(in crate::services::render) fn push_html(&mut self, html: String) -> String {
        self.push_fragment(CompatFragment::Html(html))
    }

    pub(in crate::services::render) fn push_block_html(
        &mut self,
        html: String,
    ) -> String {
        self.push_fragment(CompatFragment::BlockHtml(html))
    }

    pub(in crate::services::render) fn push_plain(&mut self, plain: &str) -> String {
        self.push_fragment(CompatFragment::Plain {
            plain: plain.to_owned(),
            html: escape_in_any_html_context(plain),
        })
    }

    fn push_fragment(&mut self, fragment: CompatFragment) -> String {
        let index = self.fragments.len();
        self.fragments.push(fragment);
        format!("{}{index}X", self.namespace)
    }

    pub(in crate::services::render) fn restore(&self, text: &str) -> String {
        let text = self.restore_block_marker_paragraphs(text);
        let data_segments = html_data_segments(&text);
        self.restore_with(&text, None, Some(&data_segments), true, |fragment| {
            match fragment {
                CompatFragment::Html(html) | CompatFragment::BlockHtml(html) => {
                    Some(html.as_str())
                }
                CompatFragment::Plain { html, .. } => Some(html.as_str()),
            }
        })
    }

    fn restore_block_marker_paragraphs(&self, text: &str) -> String {
        if self.fragments.is_empty() || !text.contains(&self.namespace) {
            return text.to_owned();
        }

        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        while let Some(relative_start) = text[cursor..].find("<p>") {
            let start = cursor + relative_start;
            let body_start = start + "<p>".len();
            let Some(relative_end) = text[body_start..].find("</p>") else {
                break;
            };
            let body_end = body_start + relative_end;
            output.push_str(&text[cursor..start]);
            let body = &text[body_start..body_end];
            if block_html_parent_is_safe(&output)
                && let Some(restored) = self.block_marker_paragraph(body)
            {
                output.push_str(&restored);
                cursor = body_end + "</p>".len();
                continue;
            }
            output.push_str(&text[start..body_end + "</p>".len()]);
            cursor = body_end + "</p>".len();
        }
        output.push_str(&text[cursor..]);
        output
    }

    fn block_marker_paragraph(&self, body: &str) -> Option<String> {
        let mut output = String::new();
        let mut cursor = 0;
        let mut count = 0;
        while cursor < body.len() {
            let rest = &body[cursor..];
            if let Some(stripped) = rest.strip_prefix("<br>") {
                cursor = body.len() - stripped.len();
                continue;
            }
            if let Some(stripped) = rest.strip_prefix("<br/>") {
                cursor = body.len() - stripped.len();
                continue;
            }
            let whitespace = rest
                .bytes()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count();
            if whitespace > 0 {
                cursor += whitespace;
                continue;
            }
            let (index, len) = self.marker_at(rest)?;
            let CompatFragment::BlockHtml(html) = &self.fragments[index] else {
                return None;
            };
            output.push_str(html);
            count += 1;
            cursor += len;
        }
        (count > 0).then_some(output)
    }

    #[cfg(test)]
    pub(in crate::services::render) fn restore_outside_html_literals(
        &self,
        text: &str,
    ) -> String {
        let data_segments = html_data_segments(text);
        self.restore_with(text, None, Some(&data_segments), true, |fragment| {
            match fragment {
                CompatFragment::Html(html) | CompatFragment::BlockHtml(html) => {
                    Some(html.as_str())
                }
                CompatFragment::Plain { html, .. } => Some(html.as_str()),
            }
        })
    }

    pub(in crate::services::render) fn restore_outside_block_html_literals(
        &self,
        text: &str,
    ) -> String {
        let literal_regions = LiteralRegionIndex::new_html_color_restoration(text);
        self.restore_with(text, Some(&literal_regions), None, true, |fragment| {
            match fragment {
                CompatFragment::Html(html) | CompatFragment::BlockHtml(html) => {
                    Some(html.as_str())
                }
                CompatFragment::Plain { html, .. } => Some(html.as_str()),
            }
        })
    }

    pub(in crate::services::render) fn restore_plain(&self, text: &str) -> String {
        self.restore_with(text, None, None, false, |fragment| match fragment {
            CompatFragment::Plain { plain, .. } => Some(plain.as_str()),
            CompatFragment::Html(_) | CompatFragment::BlockHtml(_) => None,
        })
    }

    fn restore_with<'a>(
        &'a self,
        text: &str,
        literal_regions: Option<&LiteralRegionIndex>,
        html_data_segments: Option<&[HtmlDataSegment]>,
        unwrap_block_paragraphs: bool,
        value: impl Fn(&'a CompatFragment) -> Option<&'a str>,
    ) -> String {
        if self.fragments.is_empty() || !text.contains(&self.namespace) {
            return text.to_owned();
        }
        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        while let Some(offset) = text[cursor..].find(&self.namespace) {
            let start = cursor + offset;
            output.push_str(&text[cursor..start]);
            if let Some((index, len)) = self.marker_at(&text[start..]) {
                let marker_end = start + len;
                let inside_literal =
                    literal_regions.is_some_and(|regions| regions.contains(start));
                let inside_html_data = html_data_segments.is_none_or(|segments| {
                    let insertion =
                        segments.partition_point(|segment| segment.range.start <= start);
                    insertion > 0 && marker_end <= segments[insertion - 1].range.end
                });
                if inside_literal || !inside_html_data {
                    output.push_str(&text[start..marker_end]);
                    cursor = marker_end;
                    continue;
                }
                if let Some(fragment) = value(&self.fragments[index]) {
                    if unwrap_block_paragraphs
                        && matches!(&self.fragments[index], CompatFragment::BlockHtml(_))
                    {
                        if restore_block_html_from_paragraph(
                            &mut output,
                            text,
                            marker_end,
                            fragment,
                            &mut cursor,
                        ) {
                            continue;
                        }
                        if !block_html_parent_is_safe(&output) {
                            output.push_str(&text[start..marker_end]);
                            cursor = marker_end;
                            continue;
                        }
                    }
                    output.push_str(fragment);
                    cursor = start + len;
                } else {
                    output.push_str(&text[start..start + len]);
                    cursor = start + len;
                }
            } else {
                output.push_str(&self.namespace);
                cursor = start + self.namespace.len();
            }
        }
        output.push_str(&text[cursor..]);
        output
    }

    fn marker_at(&self, text: &str) -> Option<(usize, usize)> {
        let suffix = text.strip_prefix(&self.namespace)?;
        let digits = suffix.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || suffix.as_bytes().get(digits) != Some(&b'X') {
            return None;
        }
        let index = suffix[..digits].parse::<usize>().ok()?;
        (index < self.fragments.len())
            .then_some((index, self.namespace.len() + digits + 1))
    }
}

/// Restores a trusted block marker without ever nesting block HTML in the
/// paragraph FTML created for marker text. Splitting is intentionally limited
/// to a plain-text paragraph; inline element balancing belongs to the renderer,
/// not to this trust-boundary pass.
fn restore_block_html_from_paragraph(
    output: &mut String,
    text: &str,
    marker_end: usize,
    fragment: &str,
    cursor: &mut usize,
) -> bool {
    let Some(paragraph_start) = output.rfind("<p>") else {
        return false;
    };
    let leading = &output[paragraph_start + 3..];
    if !contains_only_text_and_breaks(leading) {
        return false;
    }
    let Some(paragraph_end) = text[marker_end..].find("</p>") else {
        return false;
    };
    let trailing_end = marker_end + paragraph_end;
    let trailing = &text[marker_end..trailing_end];
    if !contains_only_text_and_breaks(trailing) {
        return false;
    }
    if !block_html_parent_is_safe(&output[..paragraph_start]) {
        return false;
    }

    let leading_end = trailing_break_start(leading).unwrap_or(leading.len());
    let trailing_start = leading_break_end(trailing).unwrap_or(0);
    output.truncate(paragraph_start + 3 + leading_end);
    let leading_is_empty = output[paragraph_start + 3..].trim().is_empty();
    let trailing_is_empty = trailing[trailing_start..].trim().is_empty();
    if leading_is_empty {
        output.truncate(paragraph_start);
    } else {
        output.push_str("</p>");
    }
    output.push_str(fragment);
    if trailing_is_empty {
        *cursor = trailing_end + "</p>".len();
    } else {
        output.push_str("<p>");
        *cursor = marker_end + trailing_start;
    }
    true
}

fn contains_only_text_and_breaks(value: &str) -> bool {
    let mut rest = value;
    while let Some(start) = rest.find('<') {
        rest = &rest[start..];
        if let Some(after) = rest
            .strip_prefix("<br>")
            .or_else(|| rest.strip_prefix("<br/>"))
            .or_else(|| rest.strip_prefix("<br />"))
        {
            rest = after;
        } else {
            return false;
        }
    }
    true
}

fn trailing_break_start(value: &str) -> Option<usize> {
    let trimmed_end = value.trim_end().len();
    ["<br>", "<br/>", "<br />"]
        .into_iter()
        .find_map(|tag| value[..trimmed_end].strip_suffix(tag).map(str::len))
}

fn leading_break_end(value: &str) -> Option<usize> {
    let leading_whitespace = value.len() - value.trim_start().len();
    let rest = &value[leading_whitespace..];
    ["<br>", "<br/>", "<br />"].into_iter().find_map(|tag| {
        rest.strip_prefix(tag).map(|after| {
            let trailing_whitespace = after.len() - after.trim_start().len();
            leading_whitespace + tag.len() + trailing_whitespace
        })
    })
}

/// A trusted block fragment may enter only the root or an open block container.
/// Determine that parent from the full emitted HTML stack rather than its last
/// tag: a preceding sibling normally ends with `</p>`, which is not the current
/// parent of a following marker.
fn block_html_parent_is_safe(prefix: &str) -> bool {
    let Some(stack) = open_html_element_stack(prefix) else {
        return false;
    };
    stack
        .last()
        .is_none_or(|parent| is_safe_block_html_container(parent))
}

/// A conservative HTML stack for trusted-fragment placement. It is not an HTML
/// reserializer: malformed tags, mismatched closers, and unsupported raw-text
/// contexts fail closed, leaving the opaque marker untouched.
fn open_html_element_stack(html: &str) -> Option<Vec<String>> {
    let mut stack = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = html[cursor..].find('<') {
        let start = cursor + relative;
        match tag_kind(&html[start..]) {
            Some(kind @ (TagKind::Comment | TagKind::BogusComment)) => {
                cursor = protected_construct_end(html, start, kind)?;
                continue;
            }
            Some(TagKind::Cdata | TagKind::Declaration) => return None,
            Some(TagKind::Element { .. }) => {}
            // A literal or malformed `<` is not a tag that can contribute a
            // trustworthy parent. Failing closed prevents the hand parser
            // below from treating `< div>` as a real `<div>` and admitting a
            // block fragment beneath an actual inline ancestor.
            None => return None,
        }
        let end = html_tag_end(html, start)?;
        let raw_tag = &html[start..end];
        let tag = raw_tag.strip_prefix('<')?.strip_suffix('>')?.trim();
        if tag.starts_with('!') || tag.starts_with('?') {
            cursor = end;
            continue;
        }
        let closing = tag.starts_with('/');
        let tag = if closing { tag[1..].trim_start() } else { tag };
        let name_end = tag
            .find(|character: char| character.is_ascii_whitespace() || character == '/')
            .unwrap_or(tag.len());
        let name = tag[..name_end].to_ascii_lowercase();
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return None;
        }
        if closing {
            if stack.last().is_none_or(|open| open != &name) {
                return None;
            }
            stack.pop();
        } else if is_foreign_self_closing(&name, raw_tag) {
            cursor = end;
            continue;
        } else if OPAQUE_ELEMENTS.contains(&name.as_str()) {
            cursor = opaque_element_end(html, end, &name)?;
            continue;
        } else if !is_void_html_element(&name) {
            if tag.trim_end().ends_with('/') {
                return None;
            }
            stack.push(name);
        }
        cursor = end;
    }
    Some(stack)
}

fn html_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut cursor = start + 1;
    let mut quote = None;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b'\'' | b'"' if quote.is_none() => quote = Some(byte),
            byte if quote == Some(byte) => quote = None,
            b'>' if quote.is_none() => return Some(cursor + 1),
            _ => {}
        }
        cursor += 1;
    }
    None
}

fn is_void_html_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn is_safe_block_html_container(name: &str) -> bool {
    matches!(
        name,
        "article"
            | "aside"
            | "blockquote"
            | "body"
            | "div"
            | "footer"
            | "header"
            | "main"
            | "section"
            | "td"
    )
}

fn escape_in_any_html_context(value: &str) -> String {
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
    fn restores_only_registered_in_range_fragments_without_recursion() {
        let mut fragments = CompatHtmlFragments::new("authored source");
        let marker_shaped = format!("{COMPAT_HTML_MARKER_PREFIX}deadbeefI0X");
        let first = fragments.push_html(format!("<b>{marker_shaped}</b>"));
        let second = fragments.push_html("<i>second</i>".to_owned());
        let foreign =
            format!("{COMPAT_HTML_MARKER_PREFIX}ffffffffffffffffffffffffffffffffI0X");
        assert_eq!(
            fragments.restore(&format!("{first}|{second}|{foreign}")),
            format!("<b>{marker_shaped}</b>|<i>second</i>|{foreign}"),
        );
    }

    #[test]
    fn malformed_and_out_of_range_markers_remain_literal() {
        let mut fragments = CompatHtmlFragments::new("");
        let valid = fragments.push_html("<b>trusted</b>".to_owned());
        let malformed = format!("{}nopeX", fragments.namespace);
        let out_of_range = format!("{}9X", fragments.namespace);
        assert_eq!(fragments.restore(&valid), "<b>trusted</b>");
        assert_eq!(fragments.restore(&malformed), malformed);
        assert_eq!(fragments.restore(&out_of_range), out_of_range);
    }

    #[test]
    fn block_html_replaces_only_the_paragraph_created_for_its_marker() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<div>trusted block</div>".to_owned());

        assert_eq!(
            fragments.restore(&format!("<section><p>{marker}</p></section>")),
            "<section><div>trusted block</div></section>",
        );
        assert_eq!(
            fragments.restore(&format!("<section>{marker}</section>")),
            "<section><div>trusted block</div></section>",
        );
        assert_eq!(
            fragments.restore(&format!("<p>before {marker} after</p>")),
            "<p>before </p><div>trusted block</div><p> after</p>",
        );
        assert_eq!(
            fragments.restore(&format!("<p> \n{marker}\n </p>")),
            "<div>trusted block</div>",
        );
        assert!(
            !fragments
                .restore(&format!("<p>before {marker} after</p>"))
                .contains("<p><div")
        );
        assert_eq!(
            fragments.restore(&format!("<p>before<br>\n{marker}<br>\nafter</p>")),
            "<p>before</p><div>trusted block</div><p>after</p>",
        );
    }

    #[test]
    fn block_html_restores_after_preceding_siblings_at_root_and_in_safe_parents() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<ul><li>trusted</li></ul>".to_owned());

        assert_eq!(
            fragments.restore(&format!("<p>before</p><p>{marker}</p>")),
            "<p>before</p><ul><li>trusted</li></ul>",
        );
        assert_eq!(
            fragments.restore(&format!("<div><p>before</p><p>{marker}</p></div>")),
            "<div><p>before</p><ul><li>trusted</li></ul></div>",
        );
        assert_eq!(
            fragments.restore(&format!("<p>before</p>{marker}")),
            "<p>before</p><ul><li>trusted</li></ul>",
        );
        assert_eq!(
            fragments.restore(&format!("<div><p>before</p>{marker}</div>")),
            "<div><p>before</p><ul><li>trusted</li></ul></div>",
        );
    }

    #[test]
    fn block_html_parent_stack_ignores_closed_siblings_and_opaque_predecessors() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<ul><li>trusted</li></ul>".to_owned());

        assert_eq!(
            fragments.restore(&format!("<!-- <span> --> <p>before</p><p>{marker}</p>")),
            "<!-- <span> --> <p>before</p><ul><li>trusted</li></ul>",
        );
        assert_eq!(
            fragments.restore(&format!("<pre><span>source</span></pre><p>{marker}</p>")),
            "<pre><span>source</span></pre><ul><li>trusted</li></ul>",
        );
    }

    #[test]
    fn block_html_parent_stack_accepts_abrupt_html_comment_endings() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<ul><li>trusted</li></ul>".to_owned());

        for comment in ["<!-->", "<!--->", "</ bogus hidden>", "</1hidden>"] {
            let restored = fragments.restore(&format!("{comment}<p>{marker}</p>"));
            assert_eq!(restored, format!("{comment}<ul><li>trusted</li></ul>"));
            assert!(!restored.contains(&marker));
        }
    }

    #[test]
    fn block_html_parent_stack_fails_closed_on_malformed_tag_text() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<ul><li>trusted</li></ul>".to_owned());

        for html in [
            format!("<span>< div><p>{marker}</p></span>"),
            format!("<x:foo><p>{marker}</p></x:foo>"),
            format!("<x:foo>{marker}</x:foo>"),
        ] {
            assert_eq!(fragments.restore(&html), html);
        }
    }

    #[test]
    fn block_html_parent_stack_follows_html_and_foreign_self_closing_rules() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<ul><li>trusted</li></ul>".to_owned());

        for html in [
            format!("<span/><p>{marker}</p>"),
            format!("<span/>{marker}"),
        ] {
            assert_eq!(fragments.restore(&html), html);
        }
        assert_eq!(
            fragments.restore(&format!("<svg/><p>{marker}</p>")),
            "<svg/><ul><li>trusted</li></ul>",
        );
    }

    #[test]
    fn context_aware_restore_only_expands_markers_in_html_text_nodes() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_html("<b>trusted</b>".to_owned());
        let html = format!(
            "{marker}<a title=\"quoted > {marker}\">{marker}</a><!-- {marker} --><code>{marker}</code>",
        );
        assert_eq!(
            fragments.restore_outside_html_literals(&html),
            format!(
                "<b>trusted</b><a title=\"quoted > {marker}\"><b>trusted</b></a><!-- {marker} --><code>{marker}</code>",
            ),
        );
    }

    #[test]
    fn block_html_never_restores_in_attributes_comments_or_opaque_elements() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<div>trusted block</div>".to_owned());
        let html = format!(
            r#"<a title="{marker}">{marker}</a><span>{marker}</span><button>{marker}</button><h2>{marker}</h2><!-- {marker} --><code>{marker}</code><pre>{marker}</pre>"#,
        );
        assert_eq!(
            fragments.restore(&html),
            format!(
                r#"<a title="{marker}">{marker}</a><span>{marker}</span><button>{marker}</button><h2>{marker}</h2><!-- {marker} --><code>{marker}</code><pre>{marker}</pre>"#,
            ),
        );
    }

    #[test]
    fn block_html_never_uses_paragraph_unwrapping_inside_unsafe_parents() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<div>trusted block</div>".to_owned());

        for parent in ["span", "a", "button", "h2", "code", "pre"] {
            let html = format!("<{parent}><p>{marker}</p></{parent}>");
            assert_eq!(fragments.restore(&html), html, "parent: {parent}");
        }
    }

    #[test]
    fn adjacent_block_markers_split_the_paragraph_ftml_puts_around_them() {
        let mut fragments = CompatHtmlFragments::new("");
        let first = fragments.push_block_html("<div>first</div>".to_owned());
        let second = fragments.push_block_html("<div>second</div>".to_owned());

        assert_eq!(
            fragments.restore(&format!("<p>{first}<br>{second}</p>")),
            "<div>first</div><div>second</div>",
        );
    }

    #[test]
    fn color_restore_expands_inline_code_but_preserves_block_literals() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_html("<span>trusted</span>".to_owned());
        let html = format!(
            "<code class=\"wj-monospace\">{marker}</code><pre><code>{marker}</code></pre><div class=\"code\"><code>{marker}</code></div><script>{marker}</script>",
        );

        assert_eq!(
            fragments.restore_outside_block_html_literals(&html),
            format!(
                "<code class=\"wj-monospace\"><span>trusted</span></code><pre><code>{marker}</code></pre><div class=\"code\"><code>{marker}</code></div><script>{marker}</script>",
            ),
        );
    }

    #[test]
    fn restores_plain_fragments_by_destination_without_recursion() {
        let mut fragments = CompatHtmlFragments::new("");
        let forged = format!("{COMPAT_HTML_MARKER_PREFIX}deadbeefI0X");
        let marker =
            fragments.push_plain(&format!(r#"tag ] <img onerror='x'> {forged}"#));

        assert_eq!(
            fragments.restore(&marker),
            format!(
                "tag&#x20;&#x5D;&#x20;&#x3C;img&#x20;onerror&#x3D;&#x27;x&#x27;&#x3E;&#x20;{forged}"
            ),
        );
        assert_eq!(
            fragments.restore_plain(&marker),
            format!(r#"tag ] <img onerror='x'> {forged}"#),
        );
    }
}
