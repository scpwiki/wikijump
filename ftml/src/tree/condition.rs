/*
 * tree/condition.rs
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

use super::clone::string_to_owned;
use std::borrow::Cow;
use strum_macros::IntoStaticStr;

/// Representation of a single condition to determine element presence.
///
/// A list of these constitutes a full condition specification, and is
/// used in blocks like `[[iftags]]` and `[[ifcategory]]`.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ElementCondition<'t> {
    pub condition: ElementConditionType,
    pub value: Cow<'t, str>,
}

impl<'t> ElementCondition<'t> {
    /// Parse out a specification.
    ///
    /// The specification is a space separated list of strings, prefixed with
    /// either `+` or `-`.
    pub fn parse(raw_spec: &'t str) -> Vec<ElementCondition<'t>> {
        // Helper to get the value and its condition type
        fn get_spec(value: &str) -> (ElementConditionType, &str) {
            if let Some(value) = value.strip_prefix('+') {
                return (ElementConditionType::Present, value);
            }

            if let Some(value) = value.strip_prefix('-') {
                return (ElementConditionType::Absent, value);
            }

            // Implicit behavior is to check for value presence.
            (ElementConditionType::Present, value)
        }

        raw_spec
            .split(' ')
            .filter(|s| !s.is_empty())
            .map(|s| {
                let (condition, value) = get_spec(s);

                ElementCondition {
                    condition,
                    value: cow!(value),
                }
            })
            .collect()
    }

    /// Determines if this condition is satisfied.
    ///
    /// That is, if the condition is `Present`, then the given value
    /// is asserted to exist in `values`,
    /// and if the condition is `Absent`, then the given value
    /// is asserted to *not* exist in `values`.
    #[inline]
    pub fn check(&self, values: &[Cow<str>]) -> bool {
        values.contains(&self.value) == self.condition.bool_value()
    }

    /// Determines if this condition is satisfied, for a single value.
    ///
    /// See also `check()`.
    #[inline]
    pub fn check_single<S: AsRef<str>>(&self, value: S) -> bool {
        (self.value == value.as_ref()) == self.condition.bool_value()
    }

    pub fn to_owned(&self) -> ElementCondition<'static> {
        ElementCondition {
            condition: self.condition,
            value: string_to_owned(&self.value),
        }
    }
}

#[derive(
    Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum ElementConditionType {
    Present,
    Absent,
}

impl ElementConditionType {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }

    #[inline]
    pub fn bool_value(self) -> bool {
        self.into()
    }
}

impl From<ElementConditionType> for bool {
    #[inline]
    fn from(condition: ElementConditionType) -> bool {
        match condition {
            ElementConditionType::Present => true,
            ElementConditionType::Absent => false,
        }
    }
}
