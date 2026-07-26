//! Source-context guard for trusted block fragments emitted by the native-list compatibility renderer.
//!
//! The renderer may only substitute a long native list with block HTML when the
//! list markers are not literal source and are not enclosed by a Wikidot scope
//! that can render an inline or otherwise unproven parent. FTML otherwise owns
//! the source unchanged. This keeps trusted `<ul>` fragments out of inline HTML
//! contexts without teaching the fragment restorer to balance arbitrary
//! authored markup.

use std::ops::Range;

mod source_scopes;

pub(super) use source_scopes::collect_unproven_scope_ranges;

use super::literal_regions::{
    LiteralRegionIndex, TextTokenCursor, WikidotTagScan, scan_wikidot_tag,
};

/// Determines whether each physical list-marker line can be replaced by a
/// trusted block fragment.
pub(super) struct NativeListSourceContext {
    literals: LiteralRegionIndex,
    unsafe_scope_ranges: Vec<Range<usize>>,
    inline_delimiter_ranges: Vec<Range<usize>>,
}

impl NativeListSourceContext {
    pub(super) fn new(source: &str) -> Self {
        let literals = LiteralRegionIndex::new(source);
        let mut unsafe_scope_ranges = collect_inline_scope_ranges(source, &literals);
        unsafe_scope_ranges.extend(collect_unproven_scope_ranges(source, &literals));
        let mut inline_delimiter_ranges =
            collect_inline_delimiter_scope_ranges(source, &literals);
        coalesce_ranges(&mut unsafe_scope_ranges);
        coalesce_ranges(&mut inline_delimiter_ranges);
        Self {
            literals,
            unsafe_scope_ranges,
            inline_delimiter_ranges,
        }
    }

    /// A list run is eligible only when every marker line is in source that
    /// FTML will treat as an approved block-fragment position.
    pub(super) fn allows_block_run(&self, line_starts: &[usize]) -> bool {
        let Some(&run_start) = line_starts.first() else {
            return false;
        };
        line_starts.iter().all(|&offset| {
            !self.literals.contains(offset)
                && !range_contains(&self.unsafe_scope_ranges, offset)
        }) && !self
            .inline_delimiter_ranges
            .iter()
            .any(|range| range.start < run_start && run_start < range.end)
    }
}

fn collect_inline_scope_ranges(
    source: &str,
    literals: &LiteralRegionIndex,
) -> Vec<Range<usize>> {
    let bytes = source.as_bytes();
    let mut literal_cursor = literals.monotone_cursor();
    let mut text_tokens = TextTokenCursor::new(source);
    let mut open_scopes = Vec::new();
    let mut ranges = Vec::new();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        if let Some(end) = literal_cursor.containing_end(cursor) {
            cursor = end;
            continue;
        }
        if bytes.get(cursor..cursor + 2) != Some(&b"[["[..]) {
            cursor += 1;
            continue;
        }

        let mut scanned_tokens = text_tokens.clone();
        match scan_wikidot_tag(
            bytes,
            cursor,
            bytes.len(),
            true,
            true,
            &mut scanned_tokens,
        ) {
            WikidotTagScan::Complete(end) => {
                match inline_scope_tag_kind(bytes, cursor, end) {
                    Some(InlineScopeTagKind::Open(kind)) => {
                        open_scopes.push((kind, cursor))
                    }
                    Some(InlineScopeTagKind::Close(kind)) => {
                        if let Some(index) = open_scopes
                            .iter()
                            .rposition(|(open_kind, _)| *open_kind == kind)
                        {
                            let (_, open) = open_scopes.remove(index);
                            ranges.push(open..end);
                        }
                    }
                    None => {}
                }
                text_tokens = scanned_tokens;
                cursor = end;
            }
            // A malformed or incomplete head is literal source, not an open
            // inline scope. The token-boundary scanner supplies the exact
            // recovery point for malformed bracket runs.
            WikidotTagScan::Malformed { resume } => cursor = resume.max(cursor + 1),
            WikidotTagScan::Unclosed => break,
        }
    }

    for (_, open) in open_scopes {
        // A complete, valid cross-tree inline opener without a close remains
        // an inline scope to FTML. Keep the remainder native rather than
        // injecting block HTML beneath it.
        ranges.push(open..source.len());
    }
    coalesce_ranges(&mut ranges);
    ranges
}

/// Pair inline delimiters before the long-list pass turns structural list
/// source into opaque text. The normal parser can reject a delimiter once it
/// sees a list boundary; that later decision cannot make an early block marker
/// safe inside the text-shaped replacement. A false positive leaves the list
/// to FTML, while a false negative can leak the marker into inline HTML.
fn collect_inline_delimiter_scope_ranges(
    source: &str,
    literals: &LiteralRegionIndex,
) -> Vec<Range<usize>> {
    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    for (open_marker, close_marker) in [
        (b"**".as_slice(), b"**".as_slice()),
        (b"//".as_slice(), b"//".as_slice()),
        (b"__".as_slice(), b"__".as_slice()),
        (b"^^".as_slice(), b"^^".as_slice()),
        (b",,".as_slice(), b",,".as_slice()),
        (b"{{".as_slice(), b"}}".as_slice()),
        (b"--".as_slice(), b"--".as_slice()),
        (b"~~".as_slice(), b"~~".as_slice()),
        (b"##".as_slice(), b"##".as_slice()),
    ] {
        if !bytes
            .windows(open_marker.len())
            .any(|window| window == open_marker)
        {
            continue;
        }
        let mut literal_cursor = literals.monotone_cursor();
        let mut open = None;
        let mut seen_line_break = false;
        let mut cursor = 0usize;

        while cursor < bytes.len() {
            if let Some(end) = literal_cursor.containing_end(cursor) {
                if let Some(start) = open.take() {
                    // An opener crossing a literal boundary is too ambiguous
                    // for this lightweight scanner. Preserve subsequent list
                    // source rather than assuming the formatter has ended.
                    ranges.push(start..bytes.len());
                }
                seen_line_break = false;
                cursor = end;
                continue;
            }

            if open.is_some()
                && bytes.get(cursor..cursor + close_marker.len()) == Some(close_marker)
                && let Some(start) = open.take()
            {
                cursor += close_marker.len();
                ranges.push(start..cursor);
                seen_line_break = false;
                continue;
            }
            if open.is_none()
                && bytes.get(cursor..cursor + open_marker.len()) == Some(open_marker)
            {
                open = Some(cursor);
                seen_line_break = false;
                cursor += open_marker.len();
                continue;
            }

            match bytes[cursor] {
                b'\r' => {
                    cursor += usize::from(bytes.get(cursor + 1) == Some(&b'\n')) + 1;
                    if seen_line_break {
                        open = None;
                        seen_line_break = false;
                    } else {
                        seen_line_break = true;
                    }
                }
                b'\n' => {
                    cursor += 1;
                    if seen_line_break {
                        open = None;
                        seen_line_break = false;
                    } else {
                        seen_line_break = true;
                    }
                }
                b' ' | b'\t' if seen_line_break => cursor += 1,
                _ => {
                    seen_line_break = false;
                    cursor += 1;
                }
            }
        }
    }

    ranges
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InlineScopeKind {
    Size,
    Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InlineScopeTagKind {
    Open(InlineScopeKind),
    Close(InlineScopeKind),
}

fn inline_scope_tag_kind(
    bytes: &[u8],
    start: usize,
    end: usize,
) -> Option<InlineScopeTagKind> {
    let close_body_end = end.checked_sub(2)?;
    let mut cursor = start + 2;
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    let closing = bytes.get(cursor) == Some(&b'/');
    if closing {
        cursor += 1;
        while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
            cursor += 1;
        }
    }
    let name_start = cursor;
    while cursor < close_body_end
        && !matches!(bytes[cursor], b' ' | b'\t' | b'\r' | b'\n')
    {
        cursor += 1;
    }
    let name = bytes.get(name_start..cursor)?;
    if name.is_empty() {
        return None;
    }
    // FTML keeps a cross-tree inline scope open when a closing head has
    // arguments or a physical line break. Parse only the two supported names
    // here so the guard does not need a more permissive generic head parser.
    if closing
        && !bytes[cursor..close_body_end]
            .iter()
            .all(|byte| matches!(*byte, b' ' | b'\t'))
    {
        return None;
    }
    let name = name.strip_suffix(b"_").unwrap_or(name);
    let kind = if name.eq_ignore_ascii_case(b"span") {
        InlineScopeKind::Span
    } else if name.eq_ignore_ascii_case(b"size") {
        InlineScopeKind::Size
    } else {
        return None;
    };
    Some(if closing {
        InlineScopeTagKind::Close(kind)
    } else {
        InlineScopeTagKind::Open(kind)
    })
}

fn coalesce_ranges(ranges: &mut Vec<Range<usize>>) {
    ranges.sort_unstable_by_key(|range| (range.start, range.end));
    let mut write = 0usize;
    for index in 0..ranges.len() {
        if write > 0 && ranges[index].start <= ranges[write - 1].end {
            ranges[write - 1].end = ranges[write - 1].end.max(ranges[index].end);
        } else {
            ranges.swap(write, index);
            write += 1;
        }
    }
    ranges.truncate(write);
}

fn range_contains(ranges: &[Range<usize>], offset: usize) -> bool {
    let insertion = ranges.partition_point(|range| range.start <= offset);
    insertion > 0 && offset < ranges[insertion - 1].end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allows(source: &str, needle: &str) -> bool {
        let context = NativeListSourceContext::new(source);
        context.allows_block_run(&[source.find(needle).expect("test marker exists")])
    }

    #[test]
    fn blocks_complete_cross_tree_inline_scopes_but_allows_div_scopes() {
        let source = concat!(
            "[[span class=\"inline\"]]\n",
            "* blocked\n",
            "[[/span]]\n",
            "[[size 120%]]\n",
            "* also blocked\n",
            "[[/size]]\n",
            "[[div class=\"block\"]]\n",
            "* allowed\n",
            "[[/div]]\n",
        );

        assert!(!allows(source, "* blocked"));
        assert!(!allows(source, "* also blocked"));
        assert!(allows(source, "* allowed"));
    }

    #[test]
    fn blocks_paired_unproven_body_scopes_but_keeps_proven_block_scopes() {
        let cases = [
            ("hidden", "[[hidden]]\n* blocked\n[[/hidden]]\n"),
            ("invisible", "[[invisible]]\n* blocked\n[[/invisible]]\n"),
            ("bold", "[[b]]\n* blocked\n[[/b]]\n"),
            ("anchor", "[[a href=\"/target\"]]\n* blocked\n[[/a]]\n"),
            ("ruby", "[[ruby]]\n* blocked\n[[/ruby]]\n"),
            (
                "unknown",
                "[[site-specific-wrapper]]\n* blocked\n[[/site-specific-wrapper]]\n",
            ),
        ];
        for (name, source) in cases {
            assert!(!allows(source, "* blocked"), "scope: {name}");
        }

        for (name, source) in [
            ("div", "[[div class=\"top-bar\"]]\n* allowed\n[[/div]]\n"),
            ("blockquote", "[[quote]]\n* allowed\n[[/quote]]\n"),
            ("symbolic alignment", "[[=]]\n* allowed\n[[/=]]\n"),
        ] {
            assert!(allows(source, "* allowed"), "scope: {name}");
        }

        let textual_align = "[[align left]]\n* blocked\n[[/align]]\n";
        assert!(!allows(textual_align, "* blocked"));

        for (name, source) in [
            ("starred div", "[[*div]]\n* blocked\n[[/div]]\n"),
            ("scored quote", "[[quote_]]\n* blocked\n[[/quote]]\n"),
            ("scored symbolic alignment", "[[=_]]\n* blocked\n[[/=]]\n"),
        ] {
            assert!(!allows(source, "* blocked"), "scope: {name}");
        }
    }

    #[test]
    fn blocks_unclosed_known_body_owners_but_ignores_unknown_unpaired_heads() {
        for source in [
            "[[hidden]]\n* blocked\n",
            "[[hidden]]\n[[/hidden bogus]]\n* blocked\n",
            "[[div]]\n* blocked\n",
        ] {
            assert!(!allows(source, "* blocked"), "source: {source:?}");
        }

        let unknown = "[[site-specific-leaf]]\n* allowed\n";
        assert!(allows(unknown, "* allowed"));
    }

    #[test]
    fn pairs_body_scope_aliases_and_score_suffixes_by_ftml_rule_identity() {
        for (name, source) in [
            ("bold alias", "[[bold]]\n* blocked\n[[/b]]\n"),
            ("strong alias", "[[strong]]\n* blocked\n[[/b]]\n"),
            (
                "anchor score suffix",
                "[[a_ href=\"/target\"]]\n* blocked\n[[/a]]\n",
            ),
            (
                "starred anchor",
                "[[*a href=\"/target\"]]\n* blocked\n[[/a]]\n",
            ),
            (
                "starred anchor alias",
                "[[*anchor href=\"/target\"]]\n* blocked\n[[/a]]\n",
            ),
            (
                "spaced starred anchor",
                "[[* a href=\"/target\"]]\n* blocked\n[[/a]]\n",
            ),
            (
                "spaced starred anchor alias",
                "[[* anchor href=\"/target\"]]\n* blocked\n[[/a]]\n",
            ),
            (
                "header-cell alternate close",
                "[[hcell]]\n* blocked\n[[/cell]]\n",
            ),
        ] {
            assert!(!allows(source, "* blocked"), "scope: {name}");
        }

        for (name, source) in [
            ("bold alias", "[[bold]]label[[/b]]\n* allowed\n"),
            (
                "anchor score suffix",
                "[[a_ href=\"/target\"]]label[[/a]]\n* allowed\n",
            ),
            (
                "blockquote alias",
                "[[quote]]label[[/blockquote]]\n* allowed\n",
            ),
            (
                "scored div",
                "[[div_ class=\"top-bar\"]]label[[/div]]\n* allowed\n",
            ),
        ] {
            assert!(allows(source, "* allowed"), "scope: {name}");
        }
    }

    #[test]
    fn blocks_paired_inline_format_delimiters_across_list_lines() {
        let cases = [
            ("bold", "**\n* blocked\n**\n"),
            ("italics", "//\n* blocked\n//\n"),
            ("underline", "__\n* blocked\n__\n"),
            ("superscript", "^^\n* blocked\n^^\n"),
            ("subscript", ",,\n* blocked\n,,\n"),
            ("monospace", "{{\n* blocked\n}}\n"),
            ("strikethrough-dashes", "--\n* blocked\n--\n"),
            ("strikethrough-tildes", "~~\n* blocked\n~~\n"),
            ("color", "##red|\n* blocked\n##\n"),
        ];
        for (name, source) in cases {
            assert!(!allows(source, "* blocked"), "delimiter: {name}");
        }
    }

    #[test]
    fn allows_lists_after_closed_or_item_local_inline_delimiters() {
        for source in [
            "**label**\n* allowed\n",
            "* allowed -- literal --\n",
            "* allowed {{literal}}\n",
        ] {
            assert!(allows(source, "* allowed"), "source: {source:?}");
        }
    }

    #[test]
    fn blocks_nested_and_unclosed_cross_tree_inline_scopes() {
        let nested = concat!(
            "[[span]]\n",
            "[[size 120%]]\n",
            "* blocked\n",
            "[[/size]]\n",
            "[[/span]]\n",
        );
        let unclosed = "[[size 120%]]\n* blocked\n";

        assert!(!allows(nested, "* blocked"));
        assert!(!allows(unclosed, "* blocked"));
    }

    #[test]
    fn ignores_complete_same_line_inline_scope_and_stray_close() {
        let source = concat!(
            "[[/span]]\n",
            "* item [[span class=\"inline\"]]label[[/span]]\n",
            "* allowed\n",
        );

        assert!(allows(source, "* item"));
    }

    #[test]
    fn does_not_close_cross_tree_scope_with_trailing_close_arguments() {
        for invalid_close in ["[[/span bogus]]", "[[/span\n]]"] {
            let source =
                format!("[[span]]\n{invalid_close}\n* still blocked\n[[/span]]\n");

            assert!(
                !allows(&source, "* still blocked"),
                "close: {invalid_close:?}"
            );
        }
    }

    #[test]
    fn malformed_inline_scope_head_recovers_without_opening_a_scope() {
        let source = concat!(
            "[[span class=unterminated\n",
            "[[div]]\n",
            "* allowed\n",
            "[[/div]]\n",
        );

        assert!(allows(source, "* allowed"));
    }

    #[test]
    fn blocks_literal_region_lines_including_crlf_and_final_line() {
        for source in [
            "[[code]]\n* blocked\n[[/code]]\n* allowed\n",
            "@@\r\n* blocked\r\n@@\r\n* allowed\r\n",
            "[!--\n* blocked\n--]\n* allowed",
        ] {
            let context = NativeListSourceContext::new(source);
            let blocked = source.find("* blocked").expect("blocked marker exists");
            let allowed = source.find("* allowed").expect("allowed marker exists");
            assert!(!context.allows_block_run(&[blocked]), "{source:?}");
            assert!(context.allows_block_run(&[allowed]), "{source:?}");
        }
    }
}
