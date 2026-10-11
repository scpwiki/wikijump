//! Compatibility-local provenance for HTML produced by trusted runtime producers.

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use uuid::Uuid;

use super::super::RenderService;
use super::super::html_text::{
    HtmlDataSegment, OPAQUE_ELEMENTS, TagKind, html_data_segments,
    is_foreign_self_closing, opaque_element_end, protected_construct_end, tag_kind,
};
use super::super::literal_regions::LiteralRegionIndex;

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static HTML_PARENT_SCANNED_BYTES: Cell<usize> = const { Cell::new(0) };
}

#[inline]
fn record_html_parent_scanned_bytes(bytes: usize) {
    #[cfg(test)]
    HTML_PARENT_SCANNED_BYTES.with(|total| {
        total.set(total.get().saturating_add(bytes));
    });
    #[cfg(not(test))]
    let _ = bytes;
}

#[cfg(test)]
fn take_html_parent_scanned_bytes() -> usize {
    HTML_PARENT_SCANNED_BYTES.with(|total| total.replace(0))
}

pub(in crate::services::render) const COMPAT_HTML_MARKER_PREFIX: &str =
    "WIKIJUMPWIKIDOTCOMPATHTML";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(in crate::services::render) struct CompatHtmlFragments {
    namespace: String,
    fragments: Vec<CompatFragment>,
    #[serde(default)]
    exact_fragments: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CompatFragment {
    Html(String),
    BlockHtml {
        html: String,
        trim_preceding_space: bool,
        allow_span_parent: bool,
        unwrap_residual_div_paragraph_prefix: bool,
        list_pages_row_boundary: Option<ListPagesRowBoundary>,
    },
    Plain {
        plain: String,
        html: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum ListPagesRowBoundary {
    Open,
    Close,
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
            exact_fragments: Vec::new(),
        }
    }

    pub(in crate::services::render) fn push_html(&mut self, html: String) -> String {
        self.push_fragment(CompatFragment::Html(html))
    }

    pub(in crate::services::render) fn push_block_html(
        &mut self,
        html: String,
    ) -> String {
        self.push_fragment(CompatFragment::BlockHtml {
            html,
            trim_preceding_space: false,
            allow_span_parent: false,
            unwrap_residual_div_paragraph_prefix: false,
            list_pages_row_boundary: None,
        })
    }

    /// Registers trusted flow content that live Wikidot permits beneath an
    /// authored `[[span]]`. This remains deliberately narrower than generic
    /// block restoration: only a `span` parent is admitted, while links,
    /// opaque elements, and all other inline parents continue to fail closed.
    pub(in crate::services::render) fn push_block_html_allowing_span_parent(
        &mut self,
        html: String,
    ) -> String {
        self.push_fragment(CompatFragment::BlockHtml {
            html,
            trim_preceding_space: false,
            allow_span_parent: true,
            unwrap_residual_div_paragraph_prefix: false,
            list_pages_row_boundary: None,
        })
    }

    #[cfg(test)]
    pub(in crate::services::render) fn push_block_html_trimming_preceding_space(
        &mut self,
        html: String,
    ) -> String {
        self.push_fragment(CompatFragment::BlockHtml {
            html,
            trim_preceding_space: true,
            allow_span_parent: false,
            unwrap_residual_div_paragraph_prefix: false,
            list_pages_row_boundary: None,
        })
    }

    pub(in crate::services::render) fn push_list_pages_row_open(
        &mut self,
        html: String,
    ) -> String {
        self.push_fragment(CompatFragment::BlockHtml {
            html,
            trim_preceding_space: false,
            allow_span_parent: false,
            unwrap_residual_div_paragraph_prefix: false,
            list_pages_row_boundary: Some(ListPagesRowBoundary::Open),
        })
    }

    pub(in crate::services::render) fn push_list_pages_row_close(
        &mut self,
        html: String,
    ) -> String {
        self.push_fragment(CompatFragment::BlockHtml {
            html,
            trim_preceding_space: true,
            allow_span_parent: false,
            unwrap_residual_div_paragraph_prefix: false,
            list_pages_row_boundary: Some(ListPagesRowBoundary::Close),
        })
    }

    /// Registers a trusted block produced while rendering a default-summary
    /// fragment. Live Wikidot leaves a preceding unmatched `[[div]]` fragment
    /// at the flow root in this context instead of wrapping it in a paragraph.
    pub(in crate::services::render) fn push_block_html_unwrapping_residual_div_prefix(
        &mut self,
        html: String,
    ) -> String {
        self.push_fragment(CompatFragment::BlockHtml {
            html,
            trim_preceding_space: false,
            allow_span_parent: false,
            unwrap_residual_div_paragraph_prefix: true,
            list_pages_row_boundary: None,
        })
    }

    pub(in crate::services::render) fn push_plain(&mut self, plain: &str) -> String {
        self.push_fragment(CompatFragment::Plain {
            plain: plain.to_owned(),
            html: escape_in_any_html_context(plain),
        })
    }

    pub(in crate::services::render) fn is_empty(&self) -> bool {
        self.fragments.is_empty() && self.exact_fragments.is_empty()
    }

    pub(in crate::services::render) fn has_exact_fragments(&self) -> bool {
        !self.exact_fragments.is_empty()
    }

    /// Register a source-preserving marker whose byte length must remain
    /// unchanged while the delayed List-mode parser runs. This is used for
    /// nested ListPages openers/closers because generated slot ranges still
    /// point into the substituted row body.
    pub(in crate::services::render) fn push_exact_html(
        &mut self,
        marker: String,
        html: String,
    ) -> String {
        debug_assert_eq!(marker.len(), html.len());
        self.exact_fragments.push((marker.clone(), html));
        marker
    }

    fn push_fragment(&mut self, fragment: CompatFragment) -> String {
        let index = self.fragments.len();
        self.fragments.push(fragment);
        format!("{}{index}X", self.namespace)
    }

    pub(in crate::services::render) fn restore(&self, text: &str) -> String {
        let text = self.restore_exact_fragments(text);
        let text = self.restore_block_marker_paragraphs(&text);
        let data_segments = html_data_segments(&text);
        self.restore_with(&text, None, Some(&data_segments), true, |fragment| {
            match fragment {
                CompatFragment::Html(html) | CompatFragment::BlockHtml { html, .. } => {
                    Some(html.as_str())
                }
                CompatFragment::Plain { html, .. } => Some(html.as_str()),
            }
        })
    }

    pub(in crate::services::render) fn restore_exact_fragments(
        &self,
        text: &str,
    ) -> String {
        self.exact_fragments
            .iter()
            .fold(text.to_owned(), |text, (marker, html)| {
                text.replace(marker, html)
            })
    }

    fn restore_block_marker_paragraphs(&self, text: &str) -> String {
        if self.fragments.is_empty() || !text.contains(&self.namespace) {
            return text.to_owned();
        }

        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        let mut parent_stack = IncrementalHtmlElementStack::default();
        while let Some(relative_start) = text[cursor..].find("<p>") {
            let start = cursor + relative_start;
            let body_start = start + "<p>".len();
            let Some(relative_end) = text[body_start..].find("</p>") else {
                break;
            };
            let body_end = body_start + relative_end;
            output.push_str(&text[cursor..start]);
            let body = &text[body_start..body_end];
            if parent_stack.advance_and_check_parent_is_safe(&output)
                && let Some((restored, trim_preceding_space)) =
                    self.block_marker_paragraph(body)
            {
                if trim_preceding_space
                    && output
                        .as_bytes()
                        .last()
                        .is_some_and(u8::is_ascii_whitespace)
                {
                    output.pop();
                }
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

    fn block_marker_paragraph(&self, body: &str) -> Option<(String, bool)> {
        let mut output = String::new();
        let mut cursor = 0;
        let mut count = 0;
        let mut trim_preceding_space = false;
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
            let CompatFragment::BlockHtml {
                html,
                trim_preceding_space: trim,
                list_pages_row_boundary: None,
                ..
            } = &self.fragments[index]
            else {
                return None;
            };
            trim_preceding_space |= count == 0 && *trim;
            output.push_str(html);
            count += 1;
            cursor += len;
        }
        (count > 0).then_some((output, trim_preceding_space))
    }

    #[cfg(test)]
    pub(in crate::services::render) fn restore_outside_html_literals(
        &self,
        text: &str,
    ) -> String {
        let data_segments = html_data_segments(text);
        self.restore_with(text, None, Some(&data_segments), true, |fragment| {
            match fragment {
                CompatFragment::Html(html) | CompatFragment::BlockHtml { html, .. } => {
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
                CompatFragment::Html(html) | CompatFragment::BlockHtml { html, .. } => {
                    Some(html.as_str())
                }
                CompatFragment::Plain { html, .. } => Some(html.as_str()),
            }
        })
    }

    pub(in crate::services::render) fn restore_plain(&self, text: &str) -> String {
        self.restore_with(text, None, None, false, |fragment| match fragment {
            CompatFragment::Plain { plain, .. } => Some(plain.as_str()),
            CompatFragment::Html(_) | CompatFragment::BlockHtml { .. } => None,
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
        let mut parent_stack = IncrementalHtmlElementStack::default();
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
                    let row_boundary = match &self.fragments[index] {
                        CompatFragment::BlockHtml {
                            list_pages_row_boundary,
                            ..
                        } => *list_pages_row_boundary,
                        CompatFragment::Html(_) | CompatFragment::Plain { .. } => None,
                    };
                    if row_boundary == Some(ListPagesRowBoundary::Close)
                        && !parent_stack.has_list_pages_row_boundary()
                    {
                        output.push_str(&text[start..marker_end]);
                        cursor = marker_end;
                        continue;
                    }
                    if matches!(
                        &self.fragments[index],
                        CompatFragment::BlockHtml {
                            trim_preceding_space: true,
                            ..
                        }
                    ) && output
                        .as_bytes()
                        .last()
                        .is_some_and(u8::is_ascii_whitespace)
                    {
                        output.pop();
                        parent_stack.clamp_parsed_len(output.len());
                    }
                    let allow_span_parent = matches!(
                        &self.fragments[index],
                        CompatFragment::BlockHtml {
                            allow_span_parent: true,
                            ..
                        },
                    );
                    let unwrap_residual_div_paragraph_prefix = matches!(
                        &self.fragments[index],
                        CompatFragment::BlockHtml {
                            unwrap_residual_div_paragraph_prefix: true,
                            ..
                        },
                    );
                    let trim_preceding_space = matches!(
                        &self.fragments[index],
                        CompatFragment::BlockHtml {
                            trim_preceding_space: true,
                            ..
                        },
                    );
                    let restored_fragment_start = if unwrap_block_paragraphs
                        && matches!(
                            &self.fragments[index],
                            CompatFragment::BlockHtml { .. },
                        ) {
                        if let Some(fragment_start) = restore_block_html_from_paragraph(
                            &mut output,
                            text,
                            marker_end,
                            fragment,
                            &mut cursor,
                            &mut parent_stack,
                            BlockParagraphRestoreOptions {
                                unwrap_residual_div_paragraph_prefix,
                                trim_preceding_space,
                            },
                        ) {
                            Some(fragment_start)
                        } else {
                            if !parent_stack
                                .advance_and_check_parent_accepts_block_fragment(
                                    &output,
                                    allow_span_parent,
                                )
                            {
                                output.push_str(&text[start..marker_end]);
                                cursor = marker_end;
                                continue;
                            }
                            None
                        }
                    } else {
                        None
                    };
                    let fragment_start = restored_fragment_start.unwrap_or_else(|| {
                        let fragment_start = output.len();
                        output.push_str(fragment);
                        cursor = start + len;
                        fragment_start
                    });
                    match row_boundary {
                        Some(ListPagesRowBoundary::Open) => {
                            parent_stack.record_list_pages_row_boundary(
                                &output,
                                fragment_start + fragment.len(),
                            );
                        }
                        Some(ListPagesRowBoundary::Close) => {
                            parent_stack.close_list_pages_row_boundary(
                                &mut output,
                                fragment_start,
                            );
                        }
                        None => {}
                    }
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

#[derive(Clone, Copy, Debug)]
struct BlockParagraphRestoreOptions {
    unwrap_residual_div_paragraph_prefix: bool,
    trim_preceding_space: bool,
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
    parent_stack: &mut IncrementalHtmlElementStack,
    options: BlockParagraphRestoreOptions,
) -> Option<usize> {
    let paragraph_start = output.rfind("<p>")?;
    let leading = &output[paragraph_start + 3..];
    if !contains_only_text_breaks_and_balanced_inline_elements(leading) {
        return None;
    }
    let paragraph_end = text[marker_end..].find("</p>")?;
    let trailing_end = marker_end + paragraph_end;
    let trailing = &text[marker_end..trailing_end];
    if !contains_only_text_breaks_and_balanced_inline_elements(trailing) {
        return None;
    }
    if !parent_stack.advance_and_check_parent_is_safe(&output[..paragraph_start]) {
        return None;
    }

    let leading_end = trailing_break_start(leading).unwrap_or(leading.len());
    let trailing_start = leading_break_end(trailing).unwrap_or(0);
    let unwrap_leading = options.unwrap_residual_div_paragraph_prefix
        && starts_with_unmatched_residual_wikidot_div(&leading[..leading_end]);
    output.truncate(paragraph_start + 3 + leading_end);
    let leading_is_empty = output[paragraph_start + 3..].trim().is_empty();
    let trailing_is_empty = trailing[trailing_start..].trim().is_empty();
    if leading_is_empty {
        output.truncate(paragraph_start);
    } else if unwrap_leading {
        let flow_break = if output[..paragraph_start].ends_with('\n') {
            ""
        } else {
            "\n"
        };
        output.replace_range(paragraph_start..paragraph_start + 3, flow_break);
    } else {
        output.push_str("</p>");
    }
    if options.trim_preceding_space
        && output
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_whitespace)
    {
        output.pop();
        parent_stack.clamp_parsed_len(output.len());
    }
    let fragment_start = output.len();
    output.push_str(fragment);
    if trailing_is_empty {
        *cursor = trailing_end + "</p>".len();
    } else {
        output.push_str("<p>");
        *cursor = marker_end + trailing_start;
    }
    Some(fragment_start)
}

fn starts_with_unmatched_residual_wikidot_div(value: &str) -> bool {
    let value = value.trim_start();
    let Some(marker_end) = value.find("]]") else {
        return false;
    };
    let marker_end = marker_end + 2;
    let marker =
        RenderService::decode_residual_wikidot_marker_quotes(&value[..marker_end]);
    if RenderService::wikidot_residual_div_attributes(&marker).is_none() {
        return false;
    }

    !value.to_ascii_lowercase().contains("[[/div]]")
}

fn contains_only_text_breaks_and_balanced_inline_elements(value: &str) -> bool {
    let mut cursor = 0;
    let mut stack = Vec::new();
    while let Some(relative_start) = value[cursor..].find('<') {
        let start = cursor + relative_start;
        let Some(TagKind::Element { .. }) = tag_kind(&value[start..]) else {
            return false;
        };
        let Some(end) = html_tag_end(value, start) else {
            return false;
        };
        let Some(tag) = value[start..end]
            .strip_prefix('<')
            .and_then(|tag| tag.strip_suffix('>'))
        else {
            return false;
        };
        let tag = tag.trim();
        let closing = tag.starts_with('/');
        let tag = if closing { tag[1..].trim_start() } else { tag };
        let name_end = tag
            .find(|character: char| character.is_ascii_whitespace() || character == '/')
            .unwrap_or(tag.len());
        let name = tag[..name_end].to_ascii_lowercase();
        if !is_safe_paragraph_inline_element(&name) {
            return false;
        }
        if closing {
            if stack.last().is_none_or(|open| open != &name) {
                return false;
            }
            stack.pop();
        } else if is_void_html_element(&name) {
            // HTML void elements are balanced by definition.
        } else {
            if tag.trim_end().ends_with('/') {
                return false;
            }
            stack.push(name);
        }
        cursor = end;
    }
    stack.is_empty()
}

fn is_safe_paragraph_inline_element(name: &str) -> bool {
    matches!(
        name,
        "a" | "abbr"
            | "b"
            | "bdi"
            | "bdo"
            | "br"
            | "cite"
            | "code"
            | "del"
            | "em"
            | "font"
            | "i"
            | "img"
            | "ins"
            | "kbd"
            | "mark"
            | "q"
            | "s"
            | "samp"
            | "small"
            | "span"
            | "strike"
            | "strong"
            | "sub"
            | "sup"
            | "time"
            | "tt"
            | "u"
            | "var"
            | "wbr"
    )
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

#[derive(Debug)]
struct IncrementalHtmlElementStack {
    parsed: usize,
    stack: Vec<String>,
    list_pages_row_depths: Vec<usize>,
    valid: bool,
}

impl Default for IncrementalHtmlElementStack {
    fn default() -> Self {
        Self {
            parsed: 0,
            stack: Vec::new(),
            list_pages_row_depths: Vec::new(),
            valid: true,
        }
    }
}

impl IncrementalHtmlElementStack {
    fn clamp_parsed_len(&mut self, new_len: usize) {
        self.parsed = self.parsed.min(new_len);
    }

    fn has_list_pages_row_boundary(&self) -> bool {
        !self.list_pages_row_depths.is_empty()
    }

    fn record_list_pages_row_boundary(&mut self, html: &str, end: usize) {
        if self.advance(&html[..end]) {
            self.list_pages_row_depths.push(self.stack.len());
        }
    }

    fn close_list_pages_row_boundary(&mut self, html: &mut String, start: usize) {
        if !self.advance(&html[..start]) {
            return;
        }
        let Some(depth) = self.list_pages_row_depths.pop() else {
            return;
        };
        if self.stack.len() < depth {
            self.valid = false;
            return;
        }
        let mut closing = String::new();
        while self.stack.len() > depth {
            let element = self
                .stack
                .pop()
                .expect("row depth is below the current element stack");
            write!(&mut closing, "</{element}>")
                .expect("writing to a String cannot fail");
        }
        html.insert_str(start, &closing);
        self.parsed = start + closing.len();
    }

    fn advance_and_check_parent_is_safe(&mut self, html: &str) -> bool {
        self.advance_and_check_parent_accepts_block_fragment(html, false)
    }

    fn advance_and_check_parent_accepts_block_fragment(
        &mut self,
        html: &str,
        allow_span_parent: bool,
    ) -> bool {
        if self.parsed > html.len() {
            *self = Self::default();
        }
        self.advance(html)
            && self.stack.last().is_none_or(|parent| {
                is_safe_block_html_container(parent)
                    || (allow_span_parent && parent == "span")
            })
    }

    fn advance(&mut self, html: &str) -> bool {
        if self.parsed > html.len() {
            *self = Self::default();
        }
        if self.valid {
            record_html_parent_scanned_bytes(html.len().saturating_sub(self.parsed));
            self.valid =
                advance_open_html_element_stack(html, &mut self.parsed, &mut self.stack);
        }
        self.valid
    }
}

fn advance_open_html_element_stack(
    html: &str,
    cursor: &mut usize,
    stack: &mut Vec<String>,
) -> bool {
    while let Some(relative) = html[*cursor..].find('<') {
        let start = *cursor + relative;
        match tag_kind(&html[start..]) {
            Some(kind @ (TagKind::Comment | TagKind::BogusComment)) => {
                let Some(end) = protected_construct_end(html, start, kind) else {
                    return false;
                };
                *cursor = end;
                continue;
            }
            Some(TagKind::Cdata | TagKind::Declaration) => return false,
            Some(TagKind::Element { .. }) => {}
            // A literal or malformed `<` is not a tag that can contribute a
            // trustworthy parent. Failing closed prevents the hand parser
            // below from treating `< div>` as a real `<div>` and admitting a
            // block fragment beneath an actual inline ancestor.
            None => return false,
        }
        let Some(end) = html_tag_end(html, start) else {
            return false;
        };
        let raw_tag = &html[start..end];
        let Some(tag) = raw_tag
            .strip_prefix('<')
            .and_then(|tag| tag.strip_suffix('>'))
        else {
            return false;
        };
        let tag = tag.trim();
        if tag.starts_with('!') || tag.starts_with('?') {
            *cursor = end;
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
            return false;
        }
        if closing {
            if name == "p" && stack.last().is_some_and(|open| open != "p") {
                // HTML tree building implicitly closes a paragraph before a
                // block <div>; the corresponding authored </p> is then stray.
                // Keep the structural stack aligned with browser recovery so
                // a later trusted ListPages row boundary can close the div.
                *cursor = end;
                continue;
            }
            if stack.last().is_none_or(|open| open != &name) {
                return false;
            }
            stack.pop();
        } else if is_foreign_self_closing(&name, raw_tag) {
            *cursor = end;
            continue;
        } else if OPAQUE_ELEMENTS.contains(&name.as_str()) {
            let Some(opaque_end) = opaque_element_end(html, end, &name) else {
                return false;
            };
            *cursor = opaque_end;
            continue;
        } else if !is_void_html_element(&name) {
            if tag.trim_end().ends_with('/') {
                return false;
            }
            if name == "div" && stack.last().is_some_and(|open| open == "p") {
                stack.pop();
            }
            stack.push(name);
        }
        *cursor = end;
    }
    *cursor = html.len();
    true
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

/// Parents that may directly contain a restored block fragment. Table cells
/// and list items are included because Wikidot places block module output
/// (ListPages, Rate) directly in `<td>`, `<th>`, and `<li>`.
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
            | "li"
            | "main"
            | "section"
            | "td"
            | "th"
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
        assert_eq!(
            fragments.restore(&format!(
                concat!(
                    "<p>{}<br>\n",
                    r#"<span style="white-space: pre-wrap;"> </span>"#,
                    "<br>\nafter</p>",
                ),
                marker,
            )),
            concat!(
                "<div>trusted block</div><p>",
                r#"<span style="white-space: pre-wrap;"> </span>"#,
                "<br>\nafter</p>",
            ),
        );
    }

    #[test]
    fn block_html_splits_a_paragraph_after_balanced_inline_content() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<div>trusted block</div>".to_owned());

        assert_eq!(
            fragments.restore(&format!(
                r#"<p><span><a href="/safe">row</a></span><br>{marker}</p>"#,
            )),
            concat!(
                r#"<p><span><a href="/safe">row</a></span></p>"#,
                "<div>trusted block</div>",
            ),
        );
        assert_eq!(
            fragments.restore(&format!(
                r#"<p>before<br>{marker}<br>after <a href="/safe">link</a></p>"#,
            )),
            concat!(
                "<p>before</p>",
                "<div>trusted block</div>",
                r#"<p>after <a href="/safe">link</a></p>"#,
            ),
        );
        assert_eq!(
            fragments.restore(&format!("<p><span>unclosed<br>{marker}</p>")),
            format!("<p><span>unclosed<br>{marker}</p>"),
        );
        assert_eq!(
            fragments.restore(&format!("<p><script>unsafe</script><br>{marker}</p>")),
            format!("<p><script>unsafe</script><br>{marker}</p>"),
        );
    }

    #[test]
    fn opted_in_block_html_unwraps_only_an_unmatched_residual_div_prefix() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html_unwrapping_residual_div_prefix(
            "<div>trusted block</div>".to_owned(),
        );

        assert_eq!(
            fragments.restore(&format!(
                "<p>[[div class=&quot;licensebox&quot;]]<br>prefix{marker}</p>",
            )),
            concat!(
                "\n[[div class=&quot;licensebox&quot;]]<br>prefix",
                "<div>trusted block</div>",
            ),
        );
        assert_eq!(
            fragments.restore(&format!(
                "<div>preceding block</div><p>[[div class=&quot;licensebox&quot;]]{marker}</p>",
            )),
            concat!(
                "<div>preceding block</div>\n",
                "[[div class=&quot;licensebox&quot;]]",
                "<div>trusted block</div>",
            ),
        );
        for leading in [
            "plain prefix",
            "[[div class=&quot;licensebox&quot;]][[/div]]",
            "[[div onclick=&quot;unsafe()&quot;]]",
        ] {
            assert_eq!(
                fragments.restore(&format!("<p>{leading}{marker}</p>")),
                format!("<p>{leading}</p><div>trusted block</div>"),
            );
        }
    }

    #[test]
    fn block_html_preserves_an_empty_paragraph_fragment_between_text() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html("<p>\n\n</p>".to_owned());

        assert_eq!(
            fragments.restore(&format!("<p>before<br>\n{marker}<br>\nafter</p>")),
            "<p>before</p><p>\n\n</p><p>after</p>",
        );
    }

    #[test]
    fn opted_in_flow_html_restores_only_beneath_a_span_parent() {
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html_allowing_span_parent(
            "<div>trusted ListPages</div>".to_owned(),
        );

        assert_eq!(
            fragments.restore(&format!("<span>{marker}</span>")),
            "<span><div>trusted ListPages</div></span>",
        );
        assert_eq!(
            fragments.restore(&format!("<a href=\"/safe\">{marker}</a>")),
            format!("<a href=\"/safe\">{marker}</a>"),
        );
        assert_eq!(
            fragments.restore(&format!("<script>{marker}</script>")),
            format!("<script>{marker}</script>"),
        );
    }

    #[test]
    fn opted_in_block_html_trims_exactly_one_preceding_ascii_whitespace() {
        let mut fragments = CompatHtmlFragments::new("");
        let ordinary = fragments.push_block_html("</div>".to_owned());
        let trimming =
            fragments.push_block_html_trimming_preceding_space("</div>".to_owned());

        assert_eq!(
            fragments.restore(&format!("after <p>{ordinary}</p>")),
            "after </div>",
        );
        assert_eq!(
            fragments.restore(&format!("after <p>{trimming}</p>")),
            "after</div>",
        );
        assert_eq!(
            fragments.restore(&format!("after  <p>{trimming}</p>")),
            "after </div>",
        );
        assert_eq!(
            fragments.restore(&format!("after\n<p>{trimming}</p>")),
            "after</div>",
        );
        assert_eq!(
            fragments.restore(&format!("after\n\n<p>{trimming}</p>")),
            "after\n</div>",
        );
        assert_eq!(
            fragments.restore(&format!("after<p>{trimming}</p>")),
            "after</div>",
        );
    }

    #[test]
    fn list_pages_row_boundary_closes_unmatched_summary_html() {
        let mut fragments = CompatHtmlFragments::new("");
        let row_open = fragments
            .push_list_pages_row_open(r#"<div class="list-pages-item">"#.to_owned());
        let row_close = fragments.push_list_pages_row_close("</div>".to_owned());

        assert_eq!(
            fragments.restore(&format!(
                "<p>{row_open}</p><div class=\"summary-owner\"><p>FIRST</p><p>{row_close}</p>",
            )),
            concat!(
                r#"<div class="list-pages-item">"#,
                r#"<div class="summary-owner"><p>FIRST</p></div>"#,
                "</div>",
            ),
        );
    }

    #[test]
    fn list_pages_row_boundary_preserves_balanced_summary_html() {
        let mut fragments = CompatHtmlFragments::new("");
        let row_open = fragments
            .push_list_pages_row_open(r#"<div class="list-pages-item">"#.to_owned());
        let row_close = fragments.push_list_pages_row_close("</div>".to_owned());

        assert_eq!(
            fragments.restore(&format!(
                "<p>{row_open}</p><div class=\"summary-owner\">FIRST</div>\n<p>{row_close}</p>",
            )),
            concat!(
                r#"<div class="list-pages-item">"#,
                r#"<div class="summary-owner">FIRST</div>"#,
                "</div>",
            ),
        );
    }

    #[test]
    fn adjacent_list_pages_row_boundaries_restore_after_unmatched_summary_html() {
        let mut fragments = CompatHtmlFragments::new("");
        let wrapper_open =
            fragments.push_block_html(r#"<div class="list-pages-box">"#.to_owned());
        let first_open = fragments
            .push_list_pages_row_open(r#"<div class="list-pages-item">"#.to_owned());
        let first_close = fragments.push_list_pages_row_close("</div>".to_owned());
        let second_open = fragments
            .push_list_pages_row_open(r#"<div class="list-pages-item">"#.to_owned());
        let second_close = fragments.push_list_pages_row_close("</div>".to_owned());
        let wrapper_close = fragments.push_block_html("</div>".to_owned());
        let parsed = format!(
            "<p>{wrapper_open}</p><p>{first_open}</p><h1>FIRST</h1><p><div style=\"text-align: right;\"></p><p>{first_close}<br>\n{second_open}</p><h1>SECOND</h1><p>{second_close}</p><p>{wrapper_close}</p>",
        );

        let restored = fragments.restore(&parsed);
        assert!(!restored.contains(COMPAT_HTML_MARKER_PREFIX), "{restored}");
        assert_eq!(
            restored.matches(r#"<div class="list-pages-item">"#).count(),
            2
        );
        assert!(restored.ends_with("</div></div>"), "{restored}");
    }

    #[test]
    fn list_pages_row_close_without_its_open_boundary_remains_literal() {
        let mut fragments = CompatHtmlFragments::new("");
        let row_close = fragments.push_list_pages_row_close("</div>".to_owned());
        let html = format!("<p>{row_close}</p>");

        assert_eq!(fragments.restore(&html), html);
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
    fn block_html_restores_as_a_direct_child_of_header_cells_and_list_items() {
        // Anonymous Wikidot PagePreviewModule (scp-wiki, 2026-10-11): ListPages
        // and Rate output inside [[hcell]] and [[li]] is a direct child of
        // <th>/<li>, exactly like the [[cell]] (<td>) and [[div]] controls.
        let mut fragments = CompatHtmlFragments::new("");
        let marker = fragments.push_block_html(
            r#"<div class="list-pages-box"><p>row</p></div>"#.to_owned(),
        );
        let block = r#"<div class="list-pages-box"><p>row</p></div>"#;

        for (open, close) in [
            ("<table>\n<tr>\n<th>", "</th>\n</tr>\n</table>"),
            (
                r#"<table><tr><th class="namerow" colspan="2">"#,
                "</th></tr></table>",
            ),
            ("<table><tr><td>", "</td></tr></table>"),
            ("<ul>\n<li>", "</li>\n</ul>"),
            ("<ol><li>", "</li></ol>"),
        ] {
            for body in [format!("<p>{marker}</p>"), marker.clone()] {
                let restored = fragments.restore(&format!("{open}{body}{close}"));
                assert_eq!(restored, format!("{open}{block}{close}"));
                assert!(!restored.contains(COMPAT_HTML_MARKER_PREFIX), "{restored}");
            }
        }
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
    fn opaque_marker_does_not_poison_a_later_safe_block_parent() {
        let mut fragments = CompatHtmlFragments::new("");
        let opaque = fragments.push_block_html("<div>opaque</div>".to_owned());
        let safe = fragments.push_block_html("<div>safe</div>".to_owned());
        let html = format!(r#"<script>const marker = "{opaque}";</script><p>{safe}</p>"#);

        assert_eq!(
            fragments.restore(&html),
            format!(r#"<script>const marker = "{opaque}";</script><div>safe</div>"#,),
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
    fn block_parent_validation_scans_many_runtime_rows_linearly() {
        const ROWS: usize = 500;
        let mut fragments = CompatHtmlFragments::new("");
        let mut html = String::new();
        for index in 0..ROWS {
            let marker = fragments.push_block_html(format!("<div>row {index}</div>"));
            html.push_str(&format!("<p>{marker}</p>"));
        }

        take_html_parent_scanned_bytes();
        let restored = fragments.restore(&html);
        let scanned = take_html_parent_scanned_bytes();

        assert_eq!(restored.matches("<div>row ").count(), ROWS);
        assert!(
            scanned <= html.len().saturating_mul(3),
            "parent validation rescanned {scanned} bytes for {} input bytes",
            html.len(),
        );
    }

    #[test]
    fn block_parent_validation_scans_split_runtime_rows_linearly() {
        const ROWS: usize = 500;
        let mut fragments = CompatHtmlFragments::new("");
        let mut html = String::new();
        for index in 0..ROWS {
            let marker = fragments.push_block_html(format!("<div>row {index}</div>"));
            html.push_str(&format!("<p>before {marker} after</p>"));
        }

        take_html_parent_scanned_bytes();
        let restored = fragments.restore(&html);
        let scanned = take_html_parent_scanned_bytes();

        assert_eq!(restored.matches("<div>row ").count(), ROWS);
        assert!(
            scanned <= html.len().saturating_mul(3),
            "split-paragraph validation rescanned {scanned} bytes for {} input bytes",
            html.len(),
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
