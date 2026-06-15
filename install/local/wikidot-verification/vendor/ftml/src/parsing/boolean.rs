/*
 * parsing/boolean.rs
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

/// Parse a boolean string into its corresponding value.
pub fn parse_boolean<S: AsRef<str>>(s: S) -> Result<bool, NonBooleanValue> {
    const NAMES: [(&str, bool); 8] = [
        ("true", true),
        ("false", false),
        ("t", true),
        ("f", false),
        ("1", true),
        ("0", false),
        ("yes", true),
        ("no", false),
    ];

    let s = s.as_ref().trim();
    for &(name, value) in &NAMES {
        if name.eq_ignore_ascii_case(s) {
            return Ok(value);
        }
    }

    Err(NonBooleanValue)
}

/// Error value for `parse_boolean()`.
/// Returned if the given string value is not a loose boolean,
/// for instance `"yes"` or `"true"`.
#[derive(Debug)]
pub struct NonBooleanValue;
