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

//! `RevisionService` utilities module.
//!
//! This module contains helper functions to convert between `serde_json`'s
//! `Value` type and lists of strings. This is a workaround until SeaORM
//! adds full support for PostgreSQL's `ARRAY` type, which we use for a few
//! different columns.
//!
//! See also https://github.com/SeaQL/sea-orm/discussions/487

use serde_json::Value as JsonValue;

#[inline]
pub fn string_list_to_json(list: Vec<String>) -> JsonValue {
    JsonValue::Array(list.into_iter().map(JsonValue::String).collect())
}

pub fn json_to_string_list(json: &JsonValue) -> Vec<String> {
    let slice = match json {
        JsonValue::Array(slice) => slice,
        _ => panic!("JSON value not a list"),
    };

    slice.iter().map(|s| str!(s)).collect()
}

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

#[test]
fn json_list() {
    macro_rules! check {
        ($list:expr, $json:expr $(,)?) => {
            assert_eq!(
                string_list_to_json($list),
                $json,
                "Expected JSON (left) doesn't match actual (right)",
            );
        };
    }

    check!(vec![], serde_json::json!([]));
    check!(
        vec![str!("apple"), str!("banana"), str!("cherry")],
        serde_json::json!(["apple", "banana", "cherry"]),
    );
}

#[test]
fn json_equals() {
    macro_rules! check {
        ($list:expr, $json:expr, $equals:expr $(,)?) => {
            let list: &[&str] = $list;

            assert_eq!(
                string_list_equals_json(&$json, list),
                $equals,
                "Expected JSON and list to be {}",
                if $equals { "equals" } else { "not equals" },
            );
        };
    }

    check!(&[], serde_json::json!([]), true);
    check!(&[], serde_json::json!(["a"]), false);
    check!(&["a"], serde_json::json!([]), false);
    check!(&["a"], serde_json::json!(["a"]), true);
    check!(&["a"], serde_json::json!(["b"]), false);
    check!(&["a", "b", "c"], serde_json::json!(["a", "b", "c"]), true);
    check!(&["a", "b", "c"], serde_json::json!(["b", "b", "c"]), false);
}
