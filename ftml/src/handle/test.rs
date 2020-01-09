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

#[derive(Debug, Copy, Clone)]
pub struct TestHandle;

impl RemoteHandle for TestHandle {
    fn get_user_by_name(&self, name: &str) -> RemoteResult<Option<User>> {
        let user = User {
            name: Cow::Owned(str!(name)),
            id: 10000,
        };

        Ok(Some(user))
    }

    fn get_user_by_id(&self, id: u64) -> RemoteResult<Option<User>> {
        let user = User {
            name: Cow::Borrowed("SomeUserHere"),
            id,
        };

        Ok(Some(user))
    }

    fn get_page(
        &self,
        name: &str,
        args: &HashMap<&str, &str>,
    ) -> RemoteResult<Option<Cow<'static, str>>> {
        let page = format!("<PAGE '{}' #{}>", name, args.len());

        Ok(Some(Cow::Owned(page)))
    }
}
