//! Safe source projection for Wikidot ListPages content sections.

use std::ops::Range;

use super::include_attachment_owners::find_wikidot_directive_end;
use super::literal_regions::LiteralRegionIndex;

fn is_wikidot_content_separator_line(line: &str) -> bool {
    let line = line.strip_suffix('\n').unwrap_or(line);
    let line = line.strip_suffix('\r').unwrap_or(line);
    line.len() >= 4 && line.bytes().all(|character| character == b'=')
}

fn wikidot_content_section_range(wikitext: &str, section: usize) -> Option<Range<usize>> {
    if section == 0 {
        return Some(0..0);
    }

    let mut current_section = 1usize;
    let mut section_start = 0usize;
    let mut line_start = 0usize;
    for line in wikitext.split_inclusive('\n') {
        let line_end = line_start + line.len();
        if is_wikidot_content_separator_line(line) {
            if current_section == section {
                return Some(section_start..line_start);
            }
            current_section += 1;
            section_start = line_end;
        }
        line_start = line_end;
    }

    (current_section == section).then_some(section_start..wikitext.len())
}

pub(super) fn wikidot_content_section(wikitext: &str, section: Option<usize>) -> String {
    let Some(section) = section else {
        return wikitext.to_owned();
    };
    wikidot_content_section_range(wikitext, section)
        .map(|range| wikitext[range].trim_matches('\n').to_owned())
        .unwrap_or_default()
}

fn boundary_crosses_literal(
    literal_regions: &LiteralRegionIndex,
    boundary: usize,
    source_len: usize,
) -> bool {
    boundary > 0
        && boundary < source_len
        && literal_regions.contains(boundary - 1)
        && literal_regions.contains(boundary)
}

fn boundary_is_inside_wikidot_head(
    source: &str,
    literal_regions: &LiteralRegionIndex,
    boundary: usize,
) -> bool {
    let mut cursor = 0usize;
    while let Some(relative_start) = source[cursor..boundary].find("[[") {
        let start = cursor + relative_start;
        if literal_regions.contains(start) {
            cursor = start + 2;
            continue;
        }
        let Some(end) = find_wikidot_directive_end(source, start + 2, source.len())
        else {
            return true;
        };
        if boundary < end {
            return true;
        }
        cursor = end;
    }
    false
}

/// Return one source section only when include expansion cannot observe syntax context crossing either section boundary.
pub(super) fn isolate_wikidot_content_section(
    wikitext: &str,
    section: usize,
) -> Option<String> {
    let range = wikidot_content_section_range(wikitext, section)?;
    let literal_regions = LiteralRegionIndex::new_wikidot_syntax(wikitext);
    if boundary_crosses_literal(&literal_regions, range.start, wikitext.len())
        || boundary_crosses_literal(&literal_regions, range.end, wikitext.len())
    {
        return None;
    }

    let lowercase = wikitext.to_ascii_lowercase();
    if lowercase.contains("[[iftags")
        || lowercase.contains("[[/iftags")
        || wikitext.contains("[[#")
        || boundary_is_inside_wikidot_head(wikitext, &literal_regions, range.start)
        || boundary_is_inside_wikidot_head(wikitext, &literal_regions, range.end)
    {
        return None;
    }

    Some(wikitext[range].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolates_plain_sections() {
        let source = "first\n=====\nsecond\n=====\nthird\n";
        assert_eq!(
            isolate_wikidot_content_section(source, 2).as_deref(),
            Some("second\n"),
        );
        assert_eq!(wikidot_content_section(source, Some(3)), "third");
        assert_eq!(wikidot_content_section(source, Some(4)), "");
    }

    #[test]
    fn rejects_literal_context_crossing_a_section_boundary() {
        for source in [
            "[!--\n=====\n[[include component:cell]]\n--]\n",
            "[[code]]\n=====\n[[include component:cell]]\n[[/code]]\n",
            "@@raw\n=====\n[[include component:cell]]\n@@\n",
        ] {
            assert_eq!(isolate_wikidot_content_section(source, 2), None, "{source}");
        }
    }

    #[test]
    fn rejects_conditionals_and_heads_crossing_a_section_boundary() {
        assert_eq!(
            isolate_wikidot_content_section(
                "[[iftags +scp]]\n=====\nbody\n[[/iftags]]\n",
                2,
            ),
            None,
        );
        assert_eq!(
            isolate_wikidot_content_section(
                "[[include component:cell | value=\"quoted ]] closer\" |\n=====\nother=x]]\n",
                2,
            ),
            None,
        );
        assert_eq!(
            isolate_wikidot_content_section(
                "[[#if 1 | [[iftags +scp]] | text ]]\n=====\nbody\n",
                2,
            ),
            None,
        );
    }

    #[test]
    fn recognizes_only_unescaped_exact_separator_lines() {
        let source = "one\n==== \ntwo\n ====\nthree\n@@====@@\nfour\n====\r\nfive\r\n";
        assert_eq!(
            wikidot_content_section(source, Some(1)),
            "one\n==== \ntwo\n ====\nthree\n@@====@@\nfour"
        );
        assert_eq!(wikidot_content_section(source, Some(2)), "five\r");
    }

    #[test]
    fn preserves_crlf_until_after_include_expansion() {
        let source = "before\r\n====\r\n[[include component:cell]]\r\n";
        assert_eq!(
            isolate_wikidot_content_section(source, 2).as_deref(),
            Some("[[include component:cell]]\r\n"),
        );
    }
}
