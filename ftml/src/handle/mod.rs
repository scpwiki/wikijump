/*
 * handle/mod.rs
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

mod missing;
mod null;
mod test;

mod prelude {
    pub use super::Handle;
    pub use crate::data::User;
    pub use crate::{RemoteError, RemoteResult};
    pub use std::collections::HashMap;
}

use self::prelude::*;

pub use self::missing::MissingHandle;
pub use self::null::NullHandle;
pub use self::test::TestHandle;

/// Series of methods to help preprocessing of wikitext.
///
/// The intent is to allow the implementer to use a locale variable
/// to determine what copy to use.
pub trait Handle {
    /// Includes the given page, substituting the passed arguments.
    /// Returns `None` if the page doesn't exist.
    fn include_page(&self, name: &str, args: &HashMap<&str, &str>) -> RemoteResult<Option<String>>;

    /// Gets a message for designating a missing `[[include]]`.
    fn include_missing_error(&self, name: &str) -> String;

    /// Gets a message for designating too many layers of nested `[[include]]`s.
    fn include_max_depth_error(&self, max_depth: usize) -> String;
}
