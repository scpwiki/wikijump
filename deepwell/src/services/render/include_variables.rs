/*
 * services/render/include_variables.rs
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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Substituting `{$name}` variables into included wikitext.
//!
//! An `[[include]]` carries named arguments that its target reads back as
//! `{$name}`. Substitution has to happen before iftags and comment branches
//! are resolved, and any variable the include did not supply has to survive
//! the pass unexpanded, which is what the protect/unprotect pair is for.

use super::compat::text_fragments::CompatTextFragments;
use super::include_comment_branches::remove_unresolved_include_comment_branches_source_local;
use super::include_variable_iftags::resolve_include_variable_iftags;
use super::service::{
    INCLUDE_VARIABLE_CLOSE_SENTINEL, INCLUDE_VARIABLE_OPEN_SENTINEL,
    INCLUDE_VARIABLE_REGEX, MAX_INCLUDE_EXPANSION_DEPTH,
};
use ftml::data::PageInfo;
use ftml::includes::IncludeRef;
use ftml::{self};
use std::borrow::Cow;

pub(super) fn apply_include_variables(content: &mut String, include: &IncludeRef<'_>) {
    for _ in 0..MAX_INCLUDE_EXPANSION_DEPTH {
        let mut expanded = String::with_capacity(content.len());
        let mut previous_end = 0;
        let mut matched = false;
        let mut changed = false;

        for capture in INCLUDE_VARIABLE_REGEX.captures_iter(content) {
            let mtch = capture.get(0).unwrap();
            let name = &capture["name"];

            if let Some(value) = include
                .variables()
                .get(name)
                .map(|value| Cow::Borrowed(trim_include_variable_value(value)))
                .or_else(|| default_include_variable_value(name).map(Cow::Owned))
            {
                expanded.push_str(&content[previous_end..mtch.start()]);
                expanded.push_str(&value);
                previous_end = mtch.end();
                matched = true;
                changed |= value != mtch.as_str();
            }
        }

        if !matched {
            break;
        }

        expanded.push_str(&content[previous_end..]);
        *content = expanded;
        if !changed {
            break;
        }
    }
}

pub(super) fn apply_include_variables_before_resolving_iftags(
    content: &mut String,
    include: &IncludeRef<'_>,
    page_info: &PageInfo<'_>,
) {
    apply_include_variables(content, include);
    resolve_include_variable_iftags(content, include.variables(), page_info);
}

pub(super) fn prepare_include_source_variables_and_comment_branches(
    content: &mut String,
    include: &IncludeRef<'_>,
    page_info: &PageInfo<'_>,
    compat_text: &mut CompatTextFragments,
) {
    apply_include_variables_before_resolving_iftags(content, include, page_info);
    // A comment branch is local to the included source once its callsite
    // variables are bound. Remove inactive branches before recursively
    // preparing that source so their conditional and include delimiters
    // cannot pair with delimiters from sibling expansions.
    remove_unresolved_include_comment_branches_source_local(content, compat_text);
}

pub(super) fn trim_include_variable_value(value: &str) -> &str {
    value.trim_end_matches([' ', '\t', '\r', '\n'])
}

pub(super) fn default_include_variable_value(name: &str) -> Option<String> {
    match name.to_ascii_lowercase().as_str() {
        "author" => Some("%%created_by%%".to_owned()),
        "shadow" => Some("no".to_owned()),
        _ => None,
    }
}

pub(super) fn is_include_variable_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

pub(super) fn protect_include_variables(content: &mut String) {
    if !content.contains("{$") {
        return;
    }
    let protected = INCLUDE_VARIABLE_REGEX
        .replace_all(content, |capture: &regex::Captures<'_>| {
            format!(
                "{}{}{}",
                INCLUDE_VARIABLE_OPEN_SENTINEL,
                &capture["name"],
                INCLUDE_VARIABLE_CLOSE_SENTINEL,
            )
        })
        .to_string();

    *content = protected;
}

pub(super) fn unprotect_include_variables(content: &mut String) {
    *content = content
        .replace(INCLUDE_VARIABLE_OPEN_SENTINEL, "{$")
        .replace(INCLUDE_VARIABLE_CLOSE_SENTINEL, "}");
}
