/*
 * types/conversion_error.rs
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

use crate::error::prelude::*;
use ftml::layout::Layout;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub struct EnumConversionError {
    name: &'static str,
    value: String,
}

impl EnumConversionError {
    #[inline]
    pub fn new<S: Into<String>>(enum_name: &'static str, value: S) -> Self {
        EnumConversionError {
            name: enum_name,
            value: value.into(),
        }
    }
}

impl Display for EnumConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "failed to convert value '{}' to a {} enum value",
            self.value, self.name,
        )
    }
}

impl StdError for EnumConversionError {}

// Helpers for types from external crates

pub fn parse_layout(value: &str) -> StdResult<Layout, EnumConversionError> {
    match value.parse() {
        Ok(layout) => Ok(layout),
        Err(_) => Err(EnumConversionError::new("Layout", value)),
    }
}

#[test]
fn enum_conversion_error_formats_context() {
    let error = EnumConversionError::new("Example", "bad-value");

    assert_eq!(
        error.to_string(),
        "failed to convert value 'bad-value' to a Example enum value",
    );
}

#[test]
fn parse_layout_accepts_known_layouts_and_rejects_unknown_values() {
    assert_eq!(parse_layout("wikidot").unwrap(), Layout::Wikidot);

    let error = parse_layout("unknown-layout").unwrap_err();
    assert_eq!(
        error.to_string(),
        "failed to convert value 'unknown-layout' to a Layout enum value",
    );
}
