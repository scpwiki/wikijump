/*
 * services/revision/utils.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

use serde_json::Value as JsonValue;

#[inline]
pub fn string_list_to_json(list: Vec<String>) -> JsonValue {
    JsonValue::Array(list.into_iter().map(JsonValue::String).collect())
}

#[inline]
pub fn string_list_equals_json<S: AsRef<str>>(json: &JsonValue, list: &[S]) -> bool {
    let slice = match json {
        JsonValue::Array(ref slice) => slice,
        _ => panic!("JSON value not a list"),
    };

    if slice.len() != list.len() {
        return false;
    }

    for (json_value, string_value) in slice.iter().zip(list.iter()) {
        let b = string_value.as_ref();
        let a = match json_value {
            JsonValue::String(ref string) => string,
            _ => panic!("JSON list item is not a string"),
        };

        if a != b {
            return false;
        }
    }

    true
}
