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
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use unicase::UniCase;

pub use self::safe::{is_safe_attribute, SAFE_ATTRIBUTES, SAFE_ATTRIBUTE_PREFIXES};

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

    pub fn to_owned(&self) -> AttributeMap<'static> {
        let mut inner = HashMap::new();

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
