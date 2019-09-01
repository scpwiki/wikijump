/*
 * handle/null.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

#[derive(Debug, Copy, Clone)]
pub struct NullHandle;

impl RemoteHandle for NullHandle {
    fn get_user_by_name(&self, _name: &str) -> RemoteResult<Option<User>> {
        Ok(None)
    }

    fn get_user_by_id(&self, _id: u64) -> RemoteResult<Option<User>> {
        Ok(None)
    }

    fn get_page(
        &self,
        _name: &str,
        _args: &HashMap<&str, &str>,
    ) -> RemoteResult<Option<Cow<'static, str>>> {
        Ok(None)
    }
}
