/*
 * filter/include/object.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

use crate::Result;
use std::borrow::Cow;
use std::collections::HashMap;

// Trait definition

pub trait Includer {
    fn get_resource(&self, name: &str, args: &HashMap<&str, &str>) -> Result<Cow<'static, str>>;
}

// Implementations

#[derive(Debug, Clone)]
pub struct NullIncluder;

impl Includer for NullIncluder {
    fn get_resource(&self, _name: &str, _args: &HashMap<&str, &str>) -> Result<Cow<'static, str>> {
        Ok(Cow::Borrowed(""))
    }
}

#[derive(Debug, Clone)]
pub struct TestIncluder;

impl Includer for TestIncluder {
    fn get_resource(&self, name: &str, args: &HashMap<&str, &str>) -> Result<Cow<'static, str>> {
        Ok(Cow::Owned(format!("<INCLUDE '{}' #{}>", name, args.len(),)))
    }
}

#[derive(Debug, Clone)]
pub struct NotFoundIncluder;

impl Includer for NotFoundIncluder {
    fn get_resource(&self, name: &str, _args: &HashMap<&str, &str>) -> Result<Cow<'static, str>> {
        Ok(Cow::Owned(format!(
            "[[div style=\"line-height: 141%; color: #b00; padding: 1em; margin: 1em; border: 1px solid #faa;\"]]\nIncluded page \"{}\" does not exist\n[[/div]]", name,
        )))
    }
}
