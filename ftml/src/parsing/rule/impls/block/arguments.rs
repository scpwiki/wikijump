/*
 * parsing/rule/impls/block/arguments.rs
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

use crate::parsing::{parse_boolean, ParseWarning, ParseWarningKind, Parser};
use crate::tree::AttributeMap;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
use unicase::UniCase;

macro_rules! make_warn {
    ($parser:expr) => {
        $parser.make_warn(ParseWarningKind::BlockMalformedArguments)
    };
}

#[derive(Debug, Clone, Default)]
pub struct Arguments<'t> {
    inner: HashMap<UniCase<&'t str>, Cow<'t, str>>,
}

impl<'t> Arguments<'t> {
    #[inline]
    pub fn new() -> Self {
        Arguments::default()
    }

    pub fn insert(&mut self, key: &'t str, value: Cow<'t, str>) {
        let key = UniCase::ascii(key);

        self.inner.insert(key, value);
    }

    pub fn get(&mut self, key: &'t str) -> Option<Cow<'t, str>> {
        let key = UniCase::ascii(key);

        self.inner.remove(&key)
    }

    pub fn get_bool(
        &mut self,
        parser: &Parser<'_, 't>,
        key: &'t str,
    ) -> Result<Option<bool>, ParseWarning> {
        match self.get(key) {
            Some(argument) => match parse_boolean(argument) {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(make_warn!(parser)),
            },
            None => Ok(None),
        }
    }

    pub fn get_value<T: FromStr>(
        &mut self,
        parser: &Parser<'_, 't>,
        key: &'t str,
    ) -> Result<Option<T>, ParseWarning> {
        match self.get(key) {
            Some(argument) => match argument.parse() {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(make_warn!(parser)),
            },
            None => Ok(None),
        }
    }

    /// Removes the `UniCase` wrappers to produce a separate hash map of keys to values.
    ///
    /// This returns a new `HashMap` suitable for inclusion in final `Element`s.
    /// It does not clone any string allocations, as they are all borrowed
    /// (or already owned, per `Cow`).
    /// It only makes a new allocation for the new `HashMap`.
    #[inline]
    pub fn to_attribute_map(&self) -> AttributeMap<'t> {
        AttributeMap::from_arguments(&self.inner)
    }
}
