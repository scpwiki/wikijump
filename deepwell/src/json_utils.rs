/*
 * json_utils.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
//! Yes, they re-allocate data constantly and are annoying. I know.
//! It's a temporary workaround. I'm sorry and want it go away too.
//!
//! See also https://github.com/SeaQL/sea-orm/discussions/487

use serde::Serialize;
use serde_json::{Result, Value as JsonValue};

#[inline]
pub fn string_list_to_json<S>(items: &[S]) -> Result<JsonValue>
where
    S: Serialize + AsRef<str>,
{
    #[derive(Serialize, Debug)]
    struct StringList<'a, S: Serialize>(&'a [S]);

    serde_json::to_value(&StringList(items))
}

pub fn json_to_string_list(value: JsonValue) -> Result<Vec<String>> {
    #[derive(Deserialize, Debug)]
    struct StringList(Vec<String>);

    serde_json::from_value(value).map(|StringList(items)| items)
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
    use serde_json::json;

    macro_rules! check {
        ($input_list:expr, $expected_json:expr $(,)?) => {{
            // Convert to JSON and ensure that matches
            let input_list = $input_list;
            let actual_json = string_list_to_json(input_list.clone());

            assert_eq!(
                actual_json, $expected_json,
                "Actual converted JSON list doesn't match expected",
            );

            // Convert back to original and ensure that matches
            let new_list = json_to_string_list(&actual_json);

            assert_eq!(
                input_list, new_list,
                "Original list doesn't match reconverted list",
            );
        }};
    }

    check!(vec![], json!([]));
    check!(
        vec![str!("apple"), str!("banana"), str!("cherry")],
        json!(["apple", "banana", "cherry"]),
    );
}

#[test]
fn json_equals() {
    use serde_json::json;

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

    check!(&[], json!([]), true);
    check!(&[], json!(["a"]), false);
    check!(&["a"], json!([]), false);
    check!(&["a"], json!(["a"]), true);
    check!(&["a"], json!(["b"]), false);
    check!(&["a", "b", "c"], json!(["a", "b", "c"]), true);
    check!(&["a", "b", "c"], json!(["b", "b", "c"]), false);
}
