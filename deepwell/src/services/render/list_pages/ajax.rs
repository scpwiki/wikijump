/*
 * services/render/list_pages/ajax.rs
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

//! Wikidot Ajax Module Connector compatibility for ListPages requests.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;

use super::super::compat::text_fragments::CompatTextFragments;
use super::super::service::{
    MAX_WIKIDOT_AJAX_MODULE_BODY_BYTES, MAX_WIKIDOT_AJAX_MODULE_PARAMETER_BYTES,
    MAX_WIKIDOT_AJAX_MODULE_PARAMETERS,
};
use super::scanner::find_list_pages_module_matches;
use super::substitution::split_list_pages_values;

pub(in crate::services::render) const AJAX_MODULE_LITERAL_MARKER_PREFIX: &str =
    "WIKIJUMPWIKIDOTAJAXMODULELITERAL";
static AJAX_MODULE_LITERAL_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"{AJAX_MODULE_LITERAL_MARKER_PREFIX}[0-9a-f]{{32}}I(?P<text>(?:[0-9a-f]{{2}})+)X",
    ))
    .unwrap()
});

pub(in crate::services::render) fn build_wikidot_list_pages_module_source(
    module_body: String,
    parameters: &BTreeMap<String, String>,
) -> Option<String> {
    if module_body.len() > MAX_WIKIDOT_AJAX_MODULE_BODY_BYTES
        || parameters.len() > MAX_WIKIDOT_AJAX_MODULE_PARAMETERS
    {
        return None;
    }

    let mut source = String::from("[[module ListPages");
    for (key, value) in parameters {
        let normalized_key = key.to_ascii_lowercase();
        if !matches!(
            normalized_key.as_str(),
            "pagetype"
                | "page_type"
                | "page-type"
                | "category"
                | "tags"
                | "tag"
                | "parent"
                | "created_at"
                | "createdat"
                | "updated_at"
                | "updatedat"
                | "created_by"
                | "createdby"
                | "rating"
                | "score"
                | "name"
                | "fullname"
                | "full_slug"
                | "fullslug"
                | "range"
                | "order"
                | "offset"
                | "limit"
                | "perpage"
                | "per_page"
                | "separate"
                | "wrapper"
                | "rss"
                | "rsstitle"
                | "rssdescription"
                | "rsshome"
                | "rsslimit"
                | "rssonly"
        ) || value.len() > MAX_WIKIDOT_AJAX_MODULE_PARAMETER_BYTES
            || value.chars().any(|character| character.is_control())
            || value.contains("]]")
        {
            return None;
        }
        let current_page_dependent = (matches!(
            normalized_key.as_str(),
            "name" | "fullname" | "full_slug" | "fullslug"
        ) && value.trim() == "=")
            || (normalized_key == "range" && value.trim() == ".")
            || (normalized_key == "parent" && value.trim() == ".")
            || (normalized_key == "category"
                && split_list_pages_values(value)
                    .iter()
                    .any(|category| category == "."));
        if current_page_dependent {
            return None;
        }

        let (quote, quoted_value) = if !value.contains('"') {
            ('"', value.as_str())
        } else if !value.contains('\'') {
            ('\'', value.as_str())
        } else {
            return None;
        };
        source.push(' ');
        source.push_str(key);
        source.push('=');
        source.push(quote);
        source.push_str(quoted_value);
        source.push(quote);
    }
    source.push_str("]]\n");
    let body_start = source.len();
    source.push_str(&module_body);
    source.push_str("\n[[/module]]");
    if wikidot_ajax_list_pages_source_is_safe(&source) {
        return Some(source);
    }

    let literalized_body = literalize_ajax_module_markers(&module_body);
    source.truncate(body_start);
    source.push_str(&literalized_body);
    source.push_str("\n[[/module]]");
    wikidot_ajax_list_pages_source_is_safe(&source).then_some(source)
}

pub(in crate::services::render) fn protect_ajax_module_literal_markers(
    source: String,
    compat_text: &mut CompatTextFragments,
) -> String {
    if !source.contains(AJAX_MODULE_LITERAL_MARKER_PREFIX) {
        return source;
    }

    AJAX_MODULE_LITERAL_MARKER_REGEX
        .replace_all(&source, |captures: &regex::Captures<'_>| {
            let bytes = hex::decode(&captures["text"])
                .expect("marker regex accepts only complete hexadecimal bytes");
            match String::from_utf8(bytes) {
                Ok(text) => compat_text.push_escaped_html_text(&text),
                Err(_) => captures[0].to_owned(),
            }
        })
        .into_owned()
}

fn wikidot_ajax_list_pages_source_is_safe(source: &str) -> bool {
    let modules = find_list_pages_module_matches(source);
    modules.len() == 1
        && modules[0].start == 0
        && modules[0].end == source.len()
        && modules[0].runtime_safe
}

fn literalize_ajax_module_markers(body: &str) -> String {
    let lowercase = body.to_ascii_lowercase();
    let mut output = String::with_capacity(body.len());
    let mut cursor = 0;

    while let Some(relative_start) = lowercase[cursor..].find("[[") {
        let start = cursor + relative_start;
        output.push_str(&body[cursor..start]);

        let suffix = &lowercase[start..];
        let end = if suffix.starts_with("[[/module]]") {
            Some(start + "[[/module]]".len())
        } else if suffix.starts_with("[[module")
            && suffix
                .as_bytes()
                .get("[[module".len())
                .is_some_and(u8::is_ascii_whitespace)
        {
            suffix.find("]]").map(|end| start + end + 2)
        } else {
            None
        };

        let Some(end) = end else {
            output.push_str("[[");
            cursor = start + 2;
            continue;
        };
        let marker = &body[start..end];
        output.push_str(AJAX_MODULE_LITERAL_MARKER_PREFIX);
        output.push_str(&Uuid::new_v4().as_simple().to_string());
        output.push('I');
        output.push_str(&hex::encode(marker));
        output.push('X');
        cursor = end;
    }

    output.push_str(&body[cursor..]);
    output
}
