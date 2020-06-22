/*
 * preproc/include/test.rs
 *
 * ftml - Library to parse Wikidot code
 * Copyright (C) 2019-2020 Ammon Smith
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
use crate::handle::TestHandle;

const TEST_CASES: [(&str, &str); 11] = [
    ("", ""),
    ("[[include component:thingy]]", "<PAGE 'component:thingy' #0>"),
    (
        "[[include component:image-block\n  name=test.png |\n  caption=SCP-XX\n]]",
        "<PAGE 'component:image-block' #2>",
    ),
    (
        "apple [[include some-page key=value | key2=value2]] banana",
        "apple <PAGE 'some-page' #2> banana",
    ),
    (
        "A\n[[include first-page\n  name=test |\n  caption=thing |\n]]\nB\n[[include second-page]]\nC",
        "A\n<PAGE 'first-page' #2>\nB\n<PAGE 'second-page' #0>\nC",
    ),
    (
        "A\n[[include B]]\nC\n[[include D]]\nE\n[[include F]]\nG\n[[include H]]\nI\n[[include J]]\nK",
        "A\n<PAGE 'B' #0>\nC\n<PAGE 'D' #0>\nE\n<PAGE 'F' #0>\nG\n<PAGE 'H' #0>\nI\n<PAGE 'J' #0>\nK",
    ),
    (
        "[[ INCLUDE component:thing \n\n | name = ARG yes amazing thing\n with newline | ]]",
        "<PAGE 'component:thing' #1>",
    ),
    (
        "A\n[[include no-sep arg = value]]\nB",
        "A\n<PAGE 'no-sep' #1>\nB",
    ),
    (
        "A\n[[include pre-sep | arg = value]]\nB",
        "A\n<PAGE 'pre-sep' #1>\nB",
    ),
    (
        "A\n[[include post-sep arg = value | ]]\nB",
        "A\n<PAGE 'post-sep' #1>\nB",
    ),
    (
        "A\n[[include both-sep | arg = value | ]]\nB",
        "A\n<PAGE 'both-sep' #1>\nB",
    ),
];

#[test]
fn test_substitute() {
    use super::super::test::test_substitution;

    test_substitution(
        "include",
        |s| {
            substitute(s, &TestHandle);
        },
        &TEST_CASES,
    );
}
