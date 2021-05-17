/*
 * macros.rs
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

/// Alias for `Cow::Borrowed` that isn't quite as long.
macro_rules! cow {
    ($value:expr $(,)?) => {{
        use std::borrow::Cow;

        Cow::Borrowed($value)
    }};
}

/// Alias for `Element::Text` from a string slice.
macro_rules! text {
    ($value:expr $(,)?) => {{
        use crate::tree::Element;

        Element::Text(cow!($value))
    }};
}

/// Like `std::write()`, except it asserts the writing succeeded.
///
/// This is done because the only failure mode for writing to a `String`
/// would be insufficient memory, which would cause an abort anyways.
macro_rules! str_write {
    ($dest:expr, $($arg:tt)*) => {{
        use std::fmt::Write;

        write!($dest, $($arg)*).expect("Writing to string failed");
    }};
}
