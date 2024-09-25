/*
 * services/view/options.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

use unicase::UniCase;
use wikidot_path::{ArgumentSchema, ArgumentValue, PageArguments};

const PAGE_ARGUMENTS_SCHEMA: ArgumentSchema = ArgumentSchema {
    valid_keys: &[
        "edit",
        "title",
        "parentPage",
        "parent",
        "name",
        "tags",
        "noredirect",
        "norender",
        "comments",
        "discuss",
        "history",
        "offset",
        "data",
    ],
    solo_keys: &[
        "edit",
        "noredirect",
        "norender",
        "rerender",
        "comments",
        "discuss",
        "history",
    ],
};

/// Describes the operations to be done by Framerail for this page view.
///
/// `RawPageOptions` (from `wikidot_path`) contains the direct key / value
/// pairs, this struct parses them into logical flags to be processed.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PageOptions {
    pub edit: bool,
    pub title: Option<String>,
    pub parent: Option<String>,
    pub tags: Option<String>,
    pub no_redirect: bool,
    pub no_render: bool,
    pub rerender: bool,
    pub comments: bool,
    pub history: bool,
    pub offset: Option<i32>,
    pub data: String,
}

impl PageOptions {
    pub fn parse(extra: &str) -> Self {
        info!("Parsing page options: '{extra}'");

        let mut arguments = PageArguments::parse(extra, PAGE_ARGUMENTS_SCHEMA).0;
        let mut options = PageOptions::default();

        macro_rules! unicase {
            ($value:expr) => {
                &UniCase::unicode($value)
            };
        }

        macro_rules! set_bool {
            ($field:ident, $key:ident $(,)?) => {{
                if let Some((value, _)) = arguments.remove(unicase!(stringify!($key))) {
                    options.$field = to_bool(value);
                }
            }};
            ($field:ident $(,)?) => {
                set_bool!($field, $field)
            };
        }

        macro_rules! set_str {
            ($field:ident, $key:ident $(,)?) => {{
                if let Some((_, value)) = arguments.remove(unicase!(stringify!($key))) {
                    options.$field = str!(value);
                }
            }};
            ($field:ident $(,)?) => {
                set_str!($field, $field)
            };
        }

        macro_rules! set_str_opt {
            ($field:ident, $key:ident $(,)?) => {{
                if let Some((_, value)) = arguments.remove(unicase!(stringify!($key))) {
                    options.$field = Some(str!(value));
                }
            }};
            ($field:ident $(,)?) => {
                set_str_opt!($field, $field)
            };
        }

        set_bool!(edit);
        set_str_opt!(title);
        set_str_opt!(parent);
        set_str_opt!(parent, parentPage);
        set_str_opt!(tags);
        set_bool!(no_redirect, noredirect);
        set_bool!(no_render, norender);
        set_bool!(rerender);
        set_bool!(comments);
        set_bool!(comments, discuss);
        set_bool!(history);

        if let Some((value, orig)) = arguments.remove(unicase!("offset")) {
            match value {
                ArgumentValue::Integer(offset) => options.offset = Some(offset),
                _ => error!("Invalid value for offset argument: {orig}"),
            }
        }

        set_str!(data);

        // Done processing arguments
        // Now go through anything remaining and emitting warnings for them

        for (key, (value, raw)) in arguments {
            warn!("Unused argument in page path: {key} -> {value:?} ('{raw}')",);
        }

        options
    }
}

fn to_bool(value: ArgumentValue) -> bool {
    debug!("Converting argument value to plain boolean: {value:?}");

    match value {
        // Simply unwrap bool
        ArgumentValue::Boolean(b) => b,

        // Convert integer to truthy/falsey
        ArgumentValue::Integer(i) => i != 0,

        // Presence indicates a value of true
        ArgumentValue::String(_) | ArgumentValue::Null => true,
    }
}
