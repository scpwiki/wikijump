/*
 * handle/null.rs
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

/// RemoteHandle where all included pages are blank.
#[derive(Debug, Copy, Clone)]
pub struct NullHandle;

impl RemoteHandle for NullHandle {
    #[inline]
    fn include_page(
        &self,
        _name: &str,
        _args: &HashMap<&str, &str>,
    ) -> RemoteResult<Option<String>> {
        Ok(Some(str!("")))
    }

    #[inline]
    fn include_missing_error(&self, name: &str) -> String {
        // Wikitext is {{name}}, but we need to escape it.
        // So it's '{{' '{{' '{}' '}}' '}}'
        // meaning "literal {", "literal {", name, "literal }", "literal }".
        format!("No such page: '{{{{{}}}}}'", name)
    }

    #[inline]
    fn include_max_depth_error(&self, max_depth: usize) -> String {
        format!("Too many layers of includes: max depth is {}", max_depth)
    }
}
