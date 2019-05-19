/*
 * filter/test.rs
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

use super::{prefilter, postfilter, SubstituteFn};

pub fn test_substitution(filter_name: &str, substitute: SubstituteFn, tests: &[(&str, &str)]) {
    let mut string = String::new();

    for (input, expected) in tests {
        string.clear();
        string.push_str(input);

        if let Err(err) = substitute(&mut string) {
            panic!(
                "Failed to perform {} substitution test string:\n{}\nExpected:\n{}\n-----\nProduced error: {}",
                filter_name, input, expected, err
            );
        }

        assert_eq!(
            &string,
            expected,
            "Output of {} substitution test didn't match:\n    actual: {:?}\n  expected: {:?}",
            filter_name, &string, expected,
        );
    }
}

#[test]
fn test_prefilter() {
}

#[test]
fn test_postfilter() {
}
