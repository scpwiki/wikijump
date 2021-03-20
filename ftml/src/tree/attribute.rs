/*
 * tree/attribute.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Debug};
use unicase::UniCase;

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct AttributeMap<'t> {
    #[serde(flatten)]
    inner: HashMap<Cow<'t, str>, Cow<'t, str>>,
}

impl<'t> AttributeMap<'t> {
    #[inline]
    pub fn new() -> Self {
        AttributeMap::default()
    }

    pub fn from_arguments(arguments: &HashMap<UniCase<&'t str>, Cow<'t, str>>) -> Self {
        let inner = arguments
            .iter()
            .filter(|(&key, _)| is_safe_attribute(key))
            .map(|(key, value)| (cow!(key.into_inner()), Cow::clone(value)))
            .collect();

        AttributeMap { inner }
    }

    pub fn insert(&mut self, attribute: &'t str, value: Cow<'t, str>) -> bool {
        let will_insert = is_safe_attribute(UniCase::ascii(attribute));
        if will_insert {
            self.inner.insert(cow!(attribute), value);
        }

        will_insert
    }
}

impl<'t> Debug for AttributeMap<'t> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

// Attribute filtering

const SAFE_ATTRIBUTE_LIST: [&str; 84] = [
    "accept",
    "align",
    "alt",
    "autocapitalize",
    "autoplay",
    "background",
    "bgcolor",
    "border",
    "buffered",
    "checked",
    "cite",
    "class",
    "cols",
    "colspan",
    "contenteditable",
    "controls",
    "coords",
    "datetime",
    "decoding",
    "default",
    "dir",
    "dirname",
    "disabled",
    "download",
    "draggable",
    "for",
    "form",
    "headers",
    "height",
    "hidden",
    "high",
    "href",
    "hreflang",
    "id",
    "inputmode",
    "ismap",
    "itemprop",
    "kind",
    "label",
    "lang",
    "list",
    "loop",
    "low",
    "max",
    "maxlength",
    "min",
    "minlength",
    "multiple",
    "muted",
    "name",
    "optimum",
    "pattern",
    "placeholder",
    "poster",
    "preload",
    "readonly",
    "rel",
    "required",
    "reversed",
    "rows",
    "rowspan",
    "sandbox",
    "scope",
    "selected",
    "shape",
    "size",
    "sizes",
    "span",
    "spellcheck",
    "src",
    "srclang",
    "srcset",
    "start",
    "step",
    "style",
    "tabindex",
    "target",
    "title",
    "translate",
    "type",
    "usemap",
    "value",
    "width",
    "wrap",
];

const SAFE_ATTRIBUTE_PREFIXES: [&str; 1] = ["data-"];

lazy_static! {
    static ref SAFE_ATTRIBUTES: HashSet<UniCase<&'static str>> = {
        SAFE_ATTRIBUTE_LIST
            .iter()
            .map(|&attribute| UniCase::ascii(attribute))
            .collect()
    };
}

fn is_safe_attribute(attribute: UniCase<&str>) -> bool {
    if SAFE_ATTRIBUTES.contains(&attribute) {
        return true;
    }

    for prefix in &SAFE_ATTRIBUTE_PREFIXES {
        if attribute.starts_with(prefix) {
            return true;
        }
    }

    false
}
