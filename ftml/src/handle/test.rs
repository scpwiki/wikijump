/*
 * handle/test.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019-2020 Ammon Smith
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

use super::prelude::*;

/// Test handle for debugging.
#[derive(Debug, Copy, Clone)]
pub struct TestHandle;

impl Handle for TestHandle {
    fn include_page(&self, name: &str, args: &HashMap<&str, &str>) -> RemoteResult<Option<String>> {
        let include = format!("<PAGE '{}' {:?}>", name, args);

        Ok(Some(include))
    }

    fn include_missing_error(&self, _name: &str) -> String {
        unreachable!()
    }

    fn include_max_depth_error(&self, _max_depth: usize) -> String {
        unreachable!()
    }
}
