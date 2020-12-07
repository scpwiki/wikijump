/*
 * parse/test.rs
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

#[test]
fn json() {
    let log = crate::build_logger();
    let text = "**apple //banana//** cherry";
    let tokens = crate::tokenize(&log, text);
    let result = crate::parse(&log, &tokens);
    println!("{:#?}", result.value());
    println!("Errors: {:#?}", result.errors());

    let json = serde_json::to_string_pretty(&result).unwrap();
    println!("JSON:\n{}", json);
}
