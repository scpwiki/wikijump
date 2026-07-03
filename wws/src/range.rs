/*
 * range.rs
 *
 * Wilson's Web Server - Serves a zoo of user-generated content
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

/// Reject requests with more than this many ranges to limit DoS surface
const MAX_RANGES: usize = 10;

#[derive(Debug, Clone, Copy)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    pub fn len(self) -> u64 {
        debug_assert!(
            self.start <= self.end,
            "invalid byte range: start ({}) > end ({})",
            self.start,
            self.end,
        );
        self.end - self.start + 1
    }
}

#[derive(Debug)]
pub enum ParsedRange {
    // No range requested or header was malformed
    None,

    // One or more satisfiable byte ranges
    Satisfiable(Vec<ByteRange>),

    // Valid header but every range is unsatisfiable
    NotSatisfiable,
}

// Range request format reference:
// https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Range_requests#multipart_ranges
pub fn parse_range_header(value: &str, file_size: u64) -> ParsedRange {
    let spec = match value.trim().strip_prefix("bytes=") {
        Some(s) => s,
        None => return ParsedRange::None,
    };

    if file_size == 0 {
        return ParsedRange::NotSatisfiable;
    }

    let mut ranges = Vec::new();
    let mut any_part = false;

    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return ParsedRange::None;
        }
        any_part = true;

        if let Some(suffix_str) = part.strip_prefix('-') {
            // "-n" -> last n bytes
            let n: u64 = match suffix_str.trim().parse() {
                Ok(n) if n > 0 => n,
                _ => return ParsedRange::None,
            };
            let start = file_size.saturating_sub(n);
            ranges.push(ByteRange {
                start,
                end: file_size - 1,
            });
        } else if let Some((start_str, end_str)) = part.split_once('-') {
            let start: u64 = match start_str.trim().parse() {
                Ok(n) => n,
                Err(_) => return ParsedRange::None,
            };
            let end_str = end_str.trim();

            if end_str.is_empty() {
                // "n-" -> byte n to EOF
                if start >= file_size {
                    continue; // unsatisfiable, skip
                }
                ranges.push(ByteRange {
                    start,
                    end: file_size - 1,
                });
            } else {
                let end: u64 = match end_str.parse() {
                    Ok(n) => n,
                    Err(_) => return ParsedRange::None,
                };
                if start > end {
                    return ParsedRange::None;
                }
                if start >= file_size {
                    continue; // unsatisfiable, skip
                }
                ranges.push(ByteRange {
                    start,
                    end: end.min(file_size - 1),
                });
            }
        } else {
            return ParsedRange::None;
        }

        if ranges.len() > MAX_RANGES {
            error!(
                "Requested too many ranges, rejecting ({} > {})",
                ranges.len(),
                MAX_RANGES
            );
            return ParsedRange::None;
        }
    }

    if !any_part {
        return ParsedRange::None;
    }

    if ranges.is_empty() {
        ParsedRange::NotSatisfiable
    } else {
        ParsedRange::Satisfiable(ranges)
    }
}

// ------------ Range evaluation helpers ------------

// If `If-Range` is present and its ETag doesn't match, `Range` must be
// ignored and the full representation returned (RFC 9110 §13.1.5)
pub fn should_evaluate_range(headers: &axum::http::HeaderMap, etag: &str) -> bool {
    match headers.get(axum::http::header::IF_RANGE) {
        Some(val) => val.to_str().is_ok_and(|v| v.trim() == etag),
        None => true,
    }
}

pub fn evaluate_range(
    headers: &axum::http::HeaderMap,
    etag: &str,
    file_size: u64,
) -> ParsedRange {
    if !should_evaluate_range(headers, etag) {
        return ParsedRange::None;
    }

    match headers.get(axum::http::header::RANGE) {
        Some(val) => match val.to_str() {
            Ok(v) => parse_range_header(v, file_size),
            Err(_) => ParsedRange::None,
        },
        None => ParsedRange::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use axum::http::header::{IF_RANGE, RANGE};

    // ------------ Range parser tests ------------

    fn ranges(value: &str, size: u64) -> Vec<(u64, u64)> {
        match parse_range_header(value, size) {
            ParsedRange::Satisfiable(rs) => rs.iter().map(|r| (r.start, r.end)).collect(),
            _ => panic!("expected Satisfiable"),
        }
    }

    fn is_none(value: &str, size: u64) -> bool {
        matches!(parse_range_header(value, size), ParsedRange::None)
    }

    fn is_not_satisfiable(value: &str, size: u64) -> bool {
        matches!(parse_range_header(value, size), ParsedRange::NotSatisfiable)
    }

    #[test]
    fn single_range() {
        assert_eq!(ranges("bytes=0-99", 12345), vec![(0, 99)]);
    }

    #[test]
    fn open_ended() {
        assert_eq!(ranges("bytes=500-", 12345), vec![(500, 12344)]);
    }

    #[test]
    fn suffix() {
        assert_eq!(ranges("bytes=-100", 12345), vec![(12245, 12344)]);
    }

    #[test]
    fn suffix_larger_than_file() {
        assert_eq!(ranges("bytes=-99999", 100), vec![(0, 99)]);
    }

    #[test]
    fn multiple() {
        // With and without whitespace after the comma.
        assert_eq!(
            ranges("bytes=0-99, 200-299", 12345),
            vec![(0, 99), (200, 299)],
        );
        assert_eq!(
            ranges("bytes=0-99,200-299", 12345),
            vec![(0, 99), (200, 299)],
        );

        // Mixed with an open-ended range.
        assert_eq!(
            ranges("bytes=0-99,500-", 12345),
            vec![(0, 99), (500, 12344)],
        );

        // Mixed with a suffix range.
        assert_eq!(
            ranges("bytes=0-99,-100", 12345),
            vec![(0, 99), (12245, 12344)],
        );

        // Closed, open-ended, and suffix together.
        assert_eq!(
            ranges("bytes=0-99, 500-, -100", 12345),
            vec![(0, 99), (500, 12344), (12245, 12344)],
        );

        // More than two ranges.
        assert_eq!(
            ranges("bytes=0-9,20-29,40-49", 12345),
            vec![(0, 9), (20, 29), (40, 49)],
        );
    }

    #[test]
    fn clamp_end() {
        assert_eq!(ranges("bytes=0-99999", 100), vec![(0, 99)]);
    }

    #[test]
    fn not_satisfiable_past_eof() {
        assert!(is_not_satisfiable("bytes=12345-12400", 12345));
    }

    #[test]
    fn not_satisfiable_empty_file() {
        assert!(is_not_satisfiable("bytes=0-0", 0));
    }

    #[test]
    fn malformed_no_prefix() {
        assert!(is_none("blocks=0-99", 12345));
    }

    #[test]
    fn malformed_start_gt_end() {
        assert!(is_none("bytes=100-50", 12345));
    }

    #[test]
    fn malformed_empty_part() {
        assert!(is_none("bytes=0-99,,200-299", 12345));
    }

    #[test]
    fn skip_unsatisfiable_keep_good() {
        // First range is past EOF, second is valid.
        assert_eq!(ranges("bytes=99999-100000, 0-99", 12345), vec![(0, 99)]);
    }

    #[test]
    fn open_ended_start_past_eof_is_not_satisfiable() {
        assert!(is_not_satisfiable("bytes=12345-", 12345));
    }

    #[test]
    fn malformed_suffix_zero_or_non_numeric() {
        assert!(is_none("bytes=-0", 12345));
        assert!(is_none("bytes=-abc", 12345));
    }

    #[test]
    fn malformed_start_or_end_values() {
        assert!(is_none("bytes=abc-10", 12345));
        assert!(is_none("bytes=0-abc", 12345));
        assert!(is_none("bytes=0", 12345));
    }

    #[test]
    fn too_many_ranges_is_rejected() {
        assert!(is_none(
            "bytes=0-0,2-2,4-4,6-6,8-8,10-10,12-12,14-14,16-16,18-18,20-20",
            12345,
        ));
    }

    #[test]
    fn byte_range_len_is_inclusive() {
        assert_eq!(ByteRange { start: 10, end: 12 }.len(), 3);
    }

    #[test]
    fn if_range_must_match_current_etag() {
        let mut headers = HeaderMap::new();
        assert!(should_evaluate_range(&headers, "\"abc\""));

        headers.insert(IF_RANGE, "\"abc\"".parse().unwrap());
        assert!(should_evaluate_range(&headers, "\"abc\""));

        headers.insert(IF_RANGE, "\"old\"".parse().unwrap());
        assert!(!should_evaluate_range(&headers, "\"abc\""));
    }

    #[test]
    fn evaluate_range_uses_range_header_when_allowed() {
        let mut headers = HeaderMap::new();
        headers.insert(RANGE, "bytes=1-2".parse().unwrap());

        assert!(matches!(
            evaluate_range(&headers, "\"abc\"", 10),
            ParsedRange::Satisfiable(ranges)
                if ranges.len() == 1 && ranges[0].start == 1 && ranges[0].end == 2
        ));
    }

    #[test]
    fn evaluate_range_ignores_range_header_when_if_range_mismatches() {
        let mut headers = HeaderMap::new();
        headers.insert(RANGE, "bytes=1-2".parse().unwrap());
        headers.insert(IF_RANGE, "\"old\"".parse().unwrap());

        assert!(matches!(
            evaluate_range(&headers, "\"abc\"", 10),
            ParsedRange::None,
        ));
    }
}
