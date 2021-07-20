/*
 * tree/attribute/mod.rs
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

mod safe;

use super::clone::string_to_owned;
use crate::parsing::parse_boolean;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug};
use unicase::UniCase;

pub use self::safe::{
    is_safe_attribute, BOOLEAN_ATTRIBUTES, SAFE_ATTRIBUTES, SAFE_ATTRIBUTE_PREFIXES,
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
            .filter(|(&key, _)| is_safe_attribute(key))
            .filter_map(|(key, mut value)| {
                // Check for special boolean behavior
                if BOOLEAN_ATTRIBUTES.contains(key) {
                    if let Ok(boolean_value) = parse_boolean(value) {
                        // It's a boolean HTML attribute, like "checked".
                        if boolean_value {
                            // true: Have a key-only attribute
                            value = &cow!("");
                        } else {
                            // false: Exclude the key entirely
                            return None;
                        }
                    }
                }

                // Add key/value pair to map
                let key = key.into_inner().to_ascii_lowercase();

                Some((Cow::Owned(key), Cow::clone(value)))
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
    pub fn get(&self) -> &BTreeMap<Cow<'t, str>, Cow<'t, str>> {
        &self.inner
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

impl<'t> Debug for AttributeMap<'t> {
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
