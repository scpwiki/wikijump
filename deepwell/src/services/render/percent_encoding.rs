/*
 * services/render/percent_encoding.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Percent-encode one URL path segment using the RFC 3986 unreserved set.
///
/// Existing percent escapes are encoded as literal percent signs. This keeps
/// encoded separators such as `%2F` inside the segment after URL decoding.
pub(super) fn percent_encode_path_segment(segment: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut output = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                output.push(byte as char);
            }
            _ => {
                output.push('%');
                output.push(HEX[(byte >> 4) as usize] as char);
                output.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::percent_encode_path_segment;

    #[test]
    fn percent_encodes_path_segments_by_utf8_byte() {
        let cases = [
            ("simple-slug_1.2~x", "simple-slug_1.2~x"),
            ("a/b?c#d e]f", "a%2Fb%3Fc%23d%20e%5Df"),
            ("日本語/頁", "%E6%97%A5%E6%9C%AC%E8%AA%9E%2F%E9%A0%81"),
            ("already%2Fencoded", "already%252Fencoded"),
        ];

        for (input, expected) in cases {
            assert_eq!(
                percent_encode_path_segment(input),
                expected,
                "input: {input}"
            );
        }
    }

    #[test]
    fn encoded_path_segments_contain_only_unreserved_bytes_or_uppercase_escapes() {
        let input = String::from_utf8((0_u8..=127).collect()).expect("ASCII is UTF-8");
        let encoded = percent_encode_path_segment(&input);
        let bytes = encoded.as_bytes();
        let mut cursor = 0;

        while cursor < bytes.len() {
            if bytes[cursor] == b'%' {
                assert!(cursor + 2 < bytes.len());
                assert!(
                    bytes[cursor + 1].is_ascii_digit()
                        || matches!(bytes[cursor + 1], b'A'..=b'F')
                );
                assert!(
                    bytes[cursor + 2].is_ascii_digit()
                        || matches!(bytes[cursor + 2], b'A'..=b'F')
                );
                cursor += 3;
            } else {
                assert!(matches!(
                    bytes[cursor],
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~'
                ));
                cursor += 1;
            }
        }
    }
}
