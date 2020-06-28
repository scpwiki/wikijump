/*
 * handle/missing.rs
 *
 * ftml - Library to parse Wikidot code
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

/// Handle implementation where all pages are considered missing.
#[derive(Debug, Copy, Clone)]
pub struct MissingHandle;

impl Handle for MissingHandle {
    #[inline]
    fn include_page(
        &self,
        _name: &str,
        _args: &HashMap<&str, &str>,
    ) -> Result<Option<String>, String> {
        Ok(None)
    }

    fn include_missing_error(&self, name: &str) -> String {
        format!(
            "
[[div class=\"error-block\"]]
Included page \"{}\" does not exist ([[a href=\"/{}/edit\"]]created it now[[/a]])
[[/div]]
",
            name, name
        )
    }

    fn include_max_depth_error(&self, max_depth: usize) -> String {
        format!(
            "
[[div class=\"error-block\"]]
Too many nested includes. (Maximum depth is {})
[[/div]]
",
            max_depth
        )
    }
}
