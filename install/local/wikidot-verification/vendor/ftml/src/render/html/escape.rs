/*
 * render/html/escape.rs
 *
 * ftml - Library to parse Wikidot text
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

pub fn escape_char(c: char) -> Option<&'static str> {
    match c {
        '>' => Some("&gt;"),
        '<' => Some("&lt;"),
        '&' => Some("&amp;"),
        '\'' => Some("&#39;"),
        '\"' => Some("&quot;"),
        '\0' => Some(" "), // convert NULL chars to spaces
        _ => None,
    }
}

pub fn escape(buffer: &mut String, s: &str) {
    for ch in s.chars() {
        match escape_char(ch) {
            Some(s) => buffer.push_str(s),
            None => buffer.push(ch),
        }
    }
}

#[test]
fn test() {
    macro_rules! test {
        ($input:expr, $expected:expr $(,)?) => {{
            let mut buffer = String::new();
            escape(&mut buffer, $input);

            assert_eq!(&buffer, $expected, "Escaped HTML doesn't match expected");
        }};
    }

    test!("", "");
    test!("Hello, world!", "Hello, world!");
    test!("x + 3 > 19, solve for x", "x + 3 &gt; 19, solve for x");
    test!(
        "<script>alert('test');</script>",
        "&lt;script&gt;alert(&#39;test&#39;);&lt;/script&gt;",
    );
    test!(
        "S & C Plastic's location",
        "S &amp; C Plastic&#39;s location",
    );
}
