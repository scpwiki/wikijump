/*
 * services/render/wikidot_literal_regions.rs
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

/// Build a byte-indexed mask for source regions that Wikidot treats literally.
///
/// The renderer's compatibility passes use byte offsets from `regex`, so the
/// mask deliberately has `source.len() + 1` entries. Each source pass advances
/// monotonically, keeping preprocessing linear in the page size.
pub(super) fn wikidot_literal_region_mask(source: &str) -> Vec<bool> {
    let mut mask = vec![false; source.len() + 1];

    let mut line_start = 0usize;
    let mut in_code = false;
    let mut in_html = false;
    for line in source.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let marker = line.trim_start().to_ascii_lowercase();
        if marker.starts_with("[[code") {
            in_code = true;
        } else if marker.starts_with("[[/code]]") {
            in_code = false;
        }
        if marker.starts_with("[[html") {
            in_html = true;
        } else if marker.starts_with("[[/html]]") {
            in_html = false;
        }

        if in_code || in_html {
            mask[line_start..line_end].fill(true);
        }
        line_start = line_end;
    }

    let mut index = 0usize;
    let mut escape_start = None;
    while let Some(offset) = source[index..].find("@@") {
        let token_start = index + offset;
        let token_end = token_start + "@@".len();
        if let Some(start) = escape_start.take() {
            mask[start..token_end].fill(true);
        } else {
            escape_start = Some(token_start);
        }
        index = token_end;
    }
    if let Some(start) = escape_start {
        mask[start..].fill(true);
    }

    let mut index = 0usize;
    while let Some(open_offset) = source[index..].find("[!--") {
        let comment_start = index + open_offset;
        let content_start = comment_start + "[!--".len();
        let comment_end = source[content_start..]
            .find("--]")
            .map_or(source.len(), |close_offset| {
                content_start + close_offset + "--]".len()
            });
        mask[comment_start..comment_end].fill(true);
        index = comment_end;
    }

    mask
}

#[cfg(test)]
mod tests {
    use super::wikidot_literal_region_mask;

    fn masked(source: &str, needle: &str) -> bool {
        let offset = source.find(needle).expect("test needle exists");
        wikidot_literal_region_mask(source)[offset]
    }

    #[test]
    fn marks_code_html_escape_and_comment_contents() {
        let source = concat!(
            "[[code]]\ncode target\n[[/code]]\n",
            "[[html]]\nhtml target\n[[/html]]\n",
            "@@escape target@@\n",
            "[!-- comment target --]\n",
            "ordinary target\n",
        );

        for target in [
            "code target",
            "html target",
            "escape target",
            "comment target",
        ] {
            assert!(masked(source, target), "{target} should be literal");
        }
        assert!(!masked(source, "ordinary target"));
    }

    #[test]
    fn preserves_byte_offsets_after_unicode() {
        let source = "雪\n@@literal target@@\nordinary target";

        assert!(masked(source, "literal target"));
        assert!(!masked(source, "ordinary target"));
        assert_eq!(wikidot_literal_region_mask(source).len(), source.len() + 1);
    }

    #[test]
    fn marks_unclosed_literal_regions_through_end_of_source() {
        for source in [
            "@@target",
            "[!-- target",
            "[[code]]\ntarget",
            "[[html]]\ntarget",
        ] {
            assert!(masked(source, "target"), "{source:?}");
        }
    }
}
