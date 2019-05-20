/*
 * filter/blockquote/test.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

use super::substitute;

const TEST_CASES: [(&str, &str); 9] = [
    ("", ""),
    (
        "> alpha\nbeta\n> gamma\ndelta",
        "[[quote]]\nalpha\n[[/quote]]\nbeta\n[[quote]]\ngamma\n[[/quote]]\ndelta",
    ),
    (
        "test\n> abc\n> def\n> ghi\n>> apple\n>> banana\n>>> durian\n>> fruit list\nend",
        "test\n[[quote]]\nabc\ndef\nghi\n[[quote]]\napple\nbanana\n[[quote]]\ndurian\n[[/quote]]\nfruit list\n[[/quote]]\n[[/quote]]\nend",
    ),
    (
        ">>>> deep quote block\n>>>> contents",
        "[[quote]]\n[[quote]]\n[[quote]]\n[[quote]]\ndeep quote block\ncontents\n[[/quote]]\n[[/quote]]\n[[/quote]]\n[[/quote]]\n",
    ),
    (
        ">no space test\n> it's weird wikidot requires it\n>  extra space",
        "[[quote]]\nno space test\nit's weird wikidot requires it\nextra space\n[[/quote]]\n",
    ),
    (
        "> multiple quotes test\n\n> another block\n>> omega\n",
        "[[quote]]\nmultiple quotes test\n[[/quote]]\n\n[[quote]]\nanother block\n[[quote]]\nomega\n[[/quote]]\n[[/quote]]\n",
    ),
    (
        "this string doesn't have any quotes in it",
        "this string doesn't have any quotes in it",
    ),
    (
        "> apple\n> > fake quote\n> >> even faker\n",
        "[[quote]]\napple\n> fake quote\n>> even faker\n[[/quote]]\n",
    ),
    (
        "[[div]]\napple\n> banana\n[[/div]]\n> durian\n",
        "[[div]]\napple\n[[quote]]\nbanana\n[[/quote]]\n[[/div]]\n[[quote]]\ndurian\n[[/quote]]\n",
    ),
];

#[test]
fn test_substitute() {
    use super::test_substitution;

    test_substitution("blockquote", substitute, &TEST_CASES);
}
