/*
 * services/view/options.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

use unicase::UniCase;
use wikidot_path::{ArgumentSchema, ArgumentValue, PageArguments};

/// Every URL path argument name Wikidot recognizes here.
///
/// Membership matters beyond the names this module itself consumes: a solo key
/// such as `norender` decides whether the next segment is its value or the next
/// pair's name by looking the segment up in this list. So a module argument
/// like `tag` must appear here even though [`PageOptions`] ignores it, or
/// `/norender/tag/alpha` would parse as `norender=tag` plus `alpha=`.
pub(super) const PAGE_ARGUMENTS_SCHEMA: ArgumentSchema = ArgumentSchema {
    valid_keys: &[
        "edit",
        // Read by the viewed page's modules, not by PageOptions.
        // See services/view/module_arguments.rs.
        "tag",
        "p",
        "category",
        "title",
        "parentPage",
        "parent",
        "name",
        "tags",
        "noredirect",
        "norender",
        "debug",
        "comments",
        "discuss",
        "history",
        "offset",
        "data",
        "t",
    ],
    solo_keys: &[
        "edit",
        "noredirect",
        "norender",
        "debug",
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
    pub debug: bool,
    pub rerender: bool,
    pub comments: bool,
    pub history: bool,
    pub offset: Option<i32>,
    pub data: String,
    #[serde(default)]
    pub template: Option<i64>,
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
        set_bool!(debug);
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

        if let Some((value, original)) = arguments.remove(unicase!("t")) {
            match value {
                ArgumentValue::Integer(template_id) if template_id > 0 => {
                    options.template = Some(i64::from(template_id));
                }
                ArgumentValue::String(template_id) => {
                    options.template = template_id
                        .parse::<i64>()
                        .ok()
                        .filter(|template_id| *template_id > 0);
                    if options.template.is_none() {
                        error!("Invalid page template ID: {original}");
                    }
                }
                _ => error!("Invalid page template ID: {original}"),
            }
        }

        // Arguments addressed to the page's modules rather than to the view.
        // PageModuleArguments reads these from the same path; drop them here so
        // they are not reported as unused.
        arguments.remove(unicase!("tag"));
        arguments.remove(unicase!("p"));
        arguments.remove(unicase!("category"));

        // Done processing arguments
        // Now go through anything remaining and emitting warnings for them

        for (key, (value, raw)) in arguments {
            warn!("Unused argument in page path: {key} -> {value:?} ('{raw}')");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_options_parse_flags_strings_offsets_and_aliases() {
        let options = PageOptions::parse(
            "/edit/title/Title/parentPage/_parent/tags/a+b/noredirect/norender/false/debug/0/rerender/comments/false/discuss/history/offset/25/data/raw/t/4000000000",
        );

        assert!(options.edit);
        assert_eq!(options.title.as_deref(), Some("Title"));
        assert_eq!(options.parent.as_deref(), Some("_parent"));
        assert_eq!(options.tags.as_deref(), Some("a+b"));
        assert!(options.no_redirect);
        assert!(!options.no_render);
        assert!(!options.debug);
        assert!(options.rerender);
        assert!(options.comments);
        assert!(options.history);
        assert_eq!(options.offset, Some(25));
        assert_eq!(options.data, "raw");
        assert_eq!(options.template, Some(4_000_000_000));
    }

    #[test]
    fn page_options_ignore_invalid_offset_values() {
        let options = PageOptions::parse("/offset/not-an-integer");

        assert_eq!(options.offset, None);
    }
}
