/*
 * services/render/wikidot_class_include_variables.rs
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

use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;
use uuid::Uuid;

const SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCLASSINCLUDEVAR";

static CLASS_INCLUDE_VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\$[a-zA-Z0-9_\-]+\}").unwrap());
static SENTINEL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"{}N(?P<namespace>[0-9A-F]{{32}})I(?P<id>[0-9A-F]{{16}})X",
        regex::escape(SENTINEL_PREFIX),
    ))
    .unwrap()
});

/// Producer-owned placeholders for include variables in Wikidot class values.
///
/// A fresh namespace prevents page or runtime data from predicting a token for
/// the current render. Restoration uses one regex pass and accepts only IDs
/// registered by this instance.
#[derive(Debug, Default)]
pub(super) struct WikidotClassIncludeVariables {
    namespace: String,
    values: BTreeMap<u64, String>,
}

impl WikidotClassIncludeVariables {
    pub(super) fn protect(wikitext: &mut String) -> Self {
        if !wikitext.contains("{$") {
            return Self::default();
        }

        let namespace = fresh_namespace(wikitext);
        let mut normalized = String::with_capacity(wikitext.len());
        let mut values = BTreeMap::new();
        let mut next_id = 0u64;

        for line in wikitext.split_inclusive('\n') {
            let trimmed = line.trim_start();
            if !(trimmed.starts_with("[[div") || trimmed.starts_with("[[span"))
                || !line.contains("class=\"")
                || !line.contains("{$")
            {
                normalized.push_str(line);
                continue;
            }

            let mut line = line.to_owned();
            let mut search_start = 0usize;
            while let Some(attr_offset) = line[search_start..].find("class=\"") {
                let value_start = search_start + attr_offset + "class=\"".len();
                let Some(value_end_offset) = line[value_start..].find('"') else {
                    break;
                };
                let value_end = value_start + value_end_offset;
                let value = &line[value_start..value_end];
                let protected = CLASS_INCLUDE_VARIABLE_REGEX
                    .replace_all(value, |captures: &regex::Captures<'_>| {
                        let id = next_id;
                        next_id += 1;
                        values.insert(id, captures[0].to_owned());
                        sentinel(&namespace, id)
                    })
                    .into_owned();

                if protected != value {
                    line.replace_range(value_start..value_end, &protected);
                    search_start = value_start + protected.len();
                } else {
                    search_start = value_end + 1;
                }
            }

            normalized.push_str(&line);
        }

        if values.is_empty() {
            return Self::default();
        }

        *wikitext = normalized;
        Self { namespace, values }
    }

    pub(super) fn restore(&self, html: String) -> String {
        if self.values.is_empty() {
            return html;
        }

        SENTINEL_REGEX
            .replace_all(&html, |captures: &regex::Captures<'_>| {
                if captures["namespace"] != self.namespace {
                    return captures[0].to_owned();
                }
                let Some(id) = u64::from_str_radix(&captures["id"], 16).ok() else {
                    return captures[0].to_owned();
                };
                self.values
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| captures[0].to_owned())
            })
            .into_owned()
    }
}

fn fresh_namespace(source: &str) -> String {
    loop {
        let namespace = Uuid::new_v4().as_simple().to_string().to_ascii_uppercase();
        if !source.contains(&format!("{SENTINEL_PREFIX}N{namespace}I")) {
            return namespace;
        }
    }
}

fn sentinel(namespace: &str, id: u64) -> String {
    format!("{SENTINEL_PREFIX}N{namespace}I{id:016X}X")
}

#[cfg(test)]
mod tests {
    use super::{SENTINEL_PREFIX, WikidotClassIncludeVariables, sentinel};

    #[test]
    fn round_trips_many_variables_without_rewriting_literal_sentinels() {
        let literal_old = "WIKIJUMPWIKIDOTCLASSINCLUDEVAR0000000000000000X";
        let literal_current = concat!(
            "WIKIJUMPWIKIDOTCLASSINCLUDEVARN",
            "00000000000000000000000000000000I0000000000000000X",
        );
        let variables = (0..12)
            .map(|index| format!("{{$value{index}}}"))
            .collect::<Vec<_>>()
            .join(" ");
        let original = format!(
            "literal {literal_old} {literal_current}\n[[div class=\"{variables}\"]]\nbody\n[[/div]]\n"
        );
        let mut protected = original.clone();
        let registry = WikidotClassIncludeVariables::protect(&mut protected);

        assert_eq!(registry.values.len(), 12);
        assert_ne!(protected, original);
        assert_eq!(registry.restore(protected), original);
    }

    #[test]
    fn restores_only_the_current_namespace_and_registered_ids() {
        let mut source = "[[span class=\"{$value}\"]]body[[/span]]".to_owned();
        let registry = WikidotClassIncludeVariables::protect(&mut source);
        let tracked = sentinel(&registry.namespace, 0);
        let unknown_id = sentinel(&registry.namespace, 1);
        let foreign = sentinel("00000000000000000000000000000000", 0);
        let html = format!("{tracked} {unknown_id} {foreign}");

        assert_eq!(
            registry.restore(html),
            format!("{{$value}} {unknown_id} {foreign}"),
        );
    }

    #[test]
    fn does_nothing_without_class_include_variables() {
        let original = format!("literal {SENTINEL_PREFIX} and {{$outside}}");
        let mut source = original.clone();
        let registry = WikidotClassIncludeVariables::protect(&mut source);

        assert!(registry.values.is_empty());
        assert_eq!(source, original);
        assert_eq!(registry.restore(source), original);
    }
}
