/*
 * services/render/service/literal_regions.rs
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

/// Incrementally tracks the literal-region state at monotonically increasing
/// byte offsets. This avoids rescanning the entire source for every protected
/// link while preserving the existing Wikidot compatibility rules.
#[derive(Debug, Default)]
pub(super) struct WikidotLiteralRegionScanner {
    cursor: usize,
    line_has_content: bool,
    in_code: bool,
    in_escape: bool,
    in_html: bool,
    in_comment: bool,
}

impl WikidotLiteralRegionScanner {
    pub(super) fn is_inside(&mut self, source: &str, target: usize) -> bool {
        debug_assert!(target >= self.cursor);
        debug_assert!(source.is_char_boundary(target));

        while self.cursor < target {
            let remaining = &source[self.cursor..target];
            let character = remaining
                .chars()
                .next()
                .expect("cursor precedes the target byte offset");

            if character == '\n' {
                self.line_has_content = false;
                self.cursor += character.len_utf8();
                continue;
            }

            if !self.line_has_content {
                if character.is_whitespace() {
                    self.cursor += character.len_utf8();
                    continue;
                }

                self.line_has_content = true;
                if starts_with_ascii_case_insensitive(remaining, "[[code") {
                    self.in_code = true;
                } else if starts_with_ascii_case_insensitive(remaining, "[[/code]]") {
                    self.in_code = false;
                }

                if starts_with_ascii_case_insensitive(remaining, "[[html") {
                    self.in_html = true;
                } else if starts_with_ascii_case_insensitive(remaining, "[[/html]]") {
                    self.in_html = false;
                }
            }

            if remaining.starts_with("[!--") {
                self.in_comment = true;
                self.cursor += "[!--".len();
            } else if remaining.starts_with("--]") {
                self.in_comment = false;
                self.cursor += "--]".len();
            } else if remaining.starts_with("@@") {
                self.in_escape = !self.in_escape;
                self.cursor += "@@".len();
            } else {
                self.cursor += character.len_utf8();
            }
        }

        self.in_code || self.in_escape || self.in_html || self.in_comment
    }
}

fn starts_with_ascii_case_insensitive(value: &str, prefix: &str) -> bool {
    value
        .as_bytes()
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::WikidotLiteralRegionScanner;

    #[test]
    fn tracks_wikipedia_link_literal_regions_at_increasing_offsets() {
        let source = concat!(
            "plain [wikipedia:plain]\n",
            "  [[CoDe type=\"text\"]] [wikipedia:code]\n",
            "[[/CODE]] [wikipedia:after-code]\n",
            "@@[wikipedia:escaped]@@ [wikipedia:after-escape]\n",
            "[!-- [wikipedia:comment] --] [wikipedia:after-comment]\n",
            "[[HtMl]] [wikipedia:html]\n",
            "[[/HTML]] [wikipedia:after-html]\n",
        );
        let expected = [false, true, false, true, false, true, false, true, false];
        let mut scanner = WikidotLiteralRegionScanner::default();

        let actual = source
            .match_indices("[wikipedia:")
            .map(|(offset, _)| scanner.is_inside(source, offset))
            .collect::<Vec<_>>();

        assert_eq!(actual, expected);
    }
}
