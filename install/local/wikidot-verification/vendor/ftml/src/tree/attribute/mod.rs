/*
 * tree/attribute/mod.rs
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

mod safe;

use super::clone::string_to_owned;
use crate::id_prefix::isolate_ids;
use crate::parsing::parse_boolean;
use crate::settings::WikitextSettings;
use crate::url::normalize_href;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug};
use unicase::UniCase;

pub use self::safe::{
    BOOLEAN_ATTRIBUTES, SAFE_ATTRIBUTE_PREFIXES, SAFE_ATTRIBUTES, URL_ATTRIBUTES,
    is_safe_attribute,
};

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct AttributeMap<'t> {
    #[serde(flatten)]
    inner: BTreeMap<Cow<'t, str>, Cow<'t, str>>,
}

impl<'t> AttributeMap<'t> {
    #[inline]
    pub fn new() -> Self {
        AttributeMap::default()
    }

    pub fn from_arguments(arguments: &HashMap<UniCase<&'t str>, Cow<'t, str>>) -> Self {
        let inner = arguments
            .iter()
            .filter(|&(key, _)| is_safe_attribute(*key))
            .filter_map(|(key, value)| {
                let mut value = Cow::clone(value);

                // Check for special boolean behavior
                if BOOLEAN_ATTRIBUTES.contains(key)
                    && let Ok(boolean_value) = parse_boolean(&value)
                {
                    // It's a boolean HTML attribute, like "checked".
                    if boolean_value {
                        // true: Have a key-only attribute
                        value = cow!("");
                    } else {
                        // false: Exclude the key entirely
                        return None;
                    }
                }

                // Check for URL-sensitive attributes
                if URL_ATTRIBUTES.contains(key) {
                    let url = normalize_href(&value, None).into_owned();
                    value = Cow::Owned(url);
                }

                // Add key/value pair to map
                let key = key.into_inner().to_ascii_lowercase();
                Some((Cow::Owned(key), value))
            })
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

    #[inline]
    pub fn remove(&mut self, attribute: &str) -> Option<Cow<'t, str>> {
        self.inner.remove(attribute)
    }

    #[inline]
    pub fn get(&self) -> &BTreeMap<Cow<'t, str>, Cow<'t, str>> {
        &self.inner
    }

    pub fn isolate_id(&mut self, settings: &WikitextSettings) {
        if settings.isolate_user_ids
            && let Some(value) = self.inner.get_mut("id")
        {
            trace!("Found 'id' attribute, isolating value");
            *value = Cow::Owned(isolate_ids(value));
        }
    }

    pub fn to_owned(&self) -> AttributeMap<'static> {
        let mut inner = BTreeMap::new();

        for (key, value) in self.inner.iter() {
            let key = string_to_owned(key);
            let value = string_to_owned(value);

            inner.insert(key, value);
        }

        AttributeMap { inner }
    }
}

impl Debug for AttributeMap<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'t> From<BTreeMap<Cow<'t, str>, Cow<'t, str>>> for AttributeMap<'t> {
    #[inline]
    fn from(map: BTreeMap<Cow<'t, str>, Cow<'t, str>>) -> AttributeMap<'t> {
        AttributeMap { inner: map }
    }
}
