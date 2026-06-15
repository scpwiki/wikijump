/*
 * render/html/attributes.rs
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

use crate::tree::AttributeMap;

type AttributeValue<'a> = &'a [&'a str];
type AttributeEntry<'a> = (&'a str, AttributeValue<'a>);

/// Helper struct produced by `attr!()` to make setting attributes easier.
#[derive(Debug)]
pub struct AddedAttributes<'a> {
    pub entries: &'a [(AttributeEntry<'a>, bool)],
    pub map: Option<&'a AttributeMap<'a>>,
}

/// Inner macro for `attr!()`. Don't use directly.
///
/// This converts the various combinations of specifying a
/// particular key/value attribute pair into a tuple.
macro_rules! attr_entry {
    ($key:expr $(,)?) => {
        attr_entry!(=> $key, &[], true)
    };

    ($key:expr, $( $value:expr ),+ ) => {
        attr_entry!(=> $key, &[ $( $value ),+ ], true)
    };

    ($key:expr, $( $value:expr ),+ ; $condition:expr) => {
        attr_entry!(=> $key, &[ $( $value ),+ ], $condition)
    };

    ($key:expr, ; $condition:expr) => {
        attr_entry!(=> $key, &[], $condition)
    };

    (=> $key:expr, $value:expr, $condition:expr) => {
        (($key, $value), $condition)
    };
}

/// Inner macro for `attr!()`. Don't use directly.
///
/// This converts the presence / absence of an `AttributeMap`
/// into the appropriate `Option<AttributeMap>` value.
macro_rules! attr_map {
    ($attribute_map:expr) => {
        Some(&$attribute_map)
    };

    () => {
        None
    };
}

/// Macro to define a series of HTML attributes to add to a tag.
///
/// It permits arbitrary key/value pairs (also handling omitted values),
/// adding conditions to whether the attribute should be added, and
/// appending an `AttributeMap` after the fact, using value prepending
/// if there are key conflicts.
macro_rules! attr {
    (
        $( $key:expr $( => $( $value:expr )+ )? $( ; if $condition:expr )? ),* $(,)?
        $( ;; $attribute_map:expr $(,)? )?
    ) => {
        AddedAttributes {
            entries: &[
                $(
                    attr_entry!( $key, $( $( $value ),+ )? $( ; $condition )? )
                ),*
            ],
            map: attr_map!( $( $attribute_map )? ),
        }
    };
}

#[test]
fn attr_macro() {
    let _ = attr!();
    let _ = attr!("key");
    let _ = attr!("key" => "value");
    let _ = attr!("key" => "value" "parts");
    let _ = attr!("key" => "value" "parts" "third");
    let _ = attr!("key" => "value" "parts" "third"; if true);
    let _ = attr!("key" => "value" "parts" "third"; if true;; AttributeMap::new());
    let _ = attr!("key" => "value"; if true);
    let _ = attr!("key" => "value"; if true;; AttributeMap::new());

    let _ = attr!("key"; if true);
    let _ = attr!(
        "key1",
        "key2"; if true
    );
    let _ = attr!(
        "key1",
        "key2"; if true,
    );
    let _ = attr!(
        "key1",
        "key2"; if true,
        "key3"; if true
    );
    let _ = attr!(
        "key1",
        "key2"; if true,
        "key3"; if true,
    );
    let _ = attr!(
        "key1",
        "key2"; if true,
        "key3"; if true;;
        AttributeMap::new()
    );

    let _ = attr!(;; AttributeMap::new());
    let _ = attr!("key";; AttributeMap::new());
    let _ = attr!("key" => "value";; AttributeMap::new());

    let _ = attr!(
        "key1" => "value1",
        "key2" => "value" "2",
        "key3" => "value3"
    );

    let _ = attr!(
        "key1" => "value1",
        "key2" => "value" "2",
        "key3" => "value3",
    );

    let _ = attr!(
        "key1" => "value1"; if true,
        "key2" => "value" "2",
        "key3" => "value3"; if false,
    );

    let _ = attr!(
        "key1" => "value1"; if true,
        "key2" => "value" "2",
        "key3" => "value3"; if false;;
        AttributeMap::new()
    );

    let _ = attr!(
        "key1" => "value1"; if true,
        "key2" => "value" "2",
        "key3" => "value3"; if false;;
        AttributeMap::new(),
    );
}
