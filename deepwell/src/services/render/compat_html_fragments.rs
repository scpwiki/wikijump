//! Render-local provenance for HTML produced by trusted runtime producers.

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use uuid::Uuid;

use super::html_text::{HtmlDataSegment, html_data_segments};
use super::literal_regions::LiteralRegionIndex;

pub(super) const COMPAT_HTML_MARKER_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATHTML";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CompatHtmlFragments {
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
    pub(super) fn new(untrusted_source: &str) -> Self {
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

    pub(super) fn push_html(&mut self, html: String) -> String {
        self.push_fragment(CompatFragment::Html(html))
    }

    pub(super) fn push_block_html(&mut self, html: String) -> String {
        self.push_fragment(CompatFragment::BlockHtml(html))
    }

    pub(super) fn push_plain(&mut self, plain: &str) -> String {
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

    pub(super) fn restore(&self, text: &str) -> String {
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

    #[cfg(test)]
    pub(super) fn restore_outside_html_literals(&self, text: &str) -> String {
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

    pub(super) fn restore_outside_block_html_literals(&self, text: &str) -> String {
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

    pub(super) fn restore_plain(&self, text: &str) -> String {
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
                        if !block_html_is_direct_child_of_safe_container(
                            &output, text, marker_end,
                        ) {
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

fn block_html_is_direct_child_of_safe_container(
    output: &str,
    text: &str,
    marker_end: usize,
) -> bool {
    let Some(tag_start) = output.rfind('<') else {
        return output.trim().is_empty() && text[marker_end..].trim().is_empty();
    };
    let Some(tag_end) = output[tag_start..].find('>').map(|end| tag_start + end) else {
        return false;
    };
    if !output[tag_end + 1..].trim().is_empty() {
        return false;
    }
    let opening = output[tag_start + 1..tag_end].trim_start();
    if opening.starts_with('/') || opening.starts_with(['!', '?']) {
        return false;
    }
    let name_end = opening
        .find(|character: char| character.is_ascii_whitespace() || character == '/')
        .unwrap_or(opening.len());
    let name = opening[..name_end].to_ascii_lowercase();
    if !matches!(
        name.as_str(),
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
    ) {
        return false;
    }
    let expected_close = format!("</{name}>");
    text[marker_end..]
        .trim_start()
        .get(..expected_close.len())
        .is_some_and(|close| close.eq_ignore_ascii_case(&expected_close))
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
    if output[paragraph_start + 3..].contains('<') {
        return false;
    }
    let Some(paragraph_end) = text[marker_end..].find("</p>") else {
        return false;
    };
    let trailing_end = marker_end + paragraph_end;
    if text[marker_end..trailing_end].contains('<') {
        return false;
    }

    let leading_is_empty = output[paragraph_start + 3..].trim().is_empty();
    let trailing_is_empty = text[marker_end..trailing_end].trim().is_empty();
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
        *cursor = marker_end;
    }
    true
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
