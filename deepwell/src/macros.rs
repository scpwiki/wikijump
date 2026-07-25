/*
 * macros.rs
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

/// Like `std::write!()`, except it asserts the writing succeeded.
///
/// This is done because the only failure mode for writing to a `String`
/// would be insufficient memory, which would cause an abort anyways.
///
/// # See also
/// * [`str_writeln!`](macro.str_writeln.html)
macro_rules! str_write {
    ($dest:expr, $($arg:tt)*) => {{
        use std::fmt::Write;
        write!($dest, $($arg)*).expect("Writing to string failed");
    }};
}

/// Like `std::writeln!()`, except it asserts the writing succeeded.
///
/// This is done because the only failure mode for writing to a `String`
/// would be insufficient memory, which would cause an abort anyways.
///
/// # See also
/// * [`str_write!`](macro.str_write.html)
macro_rules! str_writeln {
    ($dest:expr, $($arg:tt)*) => {{
        use std::fmt::Write;
        writeln!($dest, $($arg)*).expect("Writing to string failed");
    }};
}

/// Convenience macro for creating a borrowed `Cow` from a string slice.
macro_rules! cow {
    ($s:expr) => {{
        use std::borrow::Cow;
        Cow::Borrowed($s.as_ref())
    }};
}

/// Convenience macro like `cow!`, but for `Option<Cow<str>>`.
macro_rules! cow_opt {
    ($s:expr) => {{
        use ref_map::OptionRefMap;
        $s.ref_map(|s| cow!(s))
    }};
}

/// Convenience macro for making borrowed string `FluentValue`s.
macro_rules! fluent_str {
    ($value:expr) => {
        FluentValue::String(cow!(&$value))
    };
}

/// Parses a string via `FromStr`
macro_rules! parse_or_raise {
    ($s:expr, $ty:ty, $raise:expr) => {
        <$ty as ::std::str::FromStr>::from_str($s).or_raise($raise)?
    };
}

/// Performs `.or_raise()?` on a number of `Result`s.
/// Intended for use in conjunction with `join!()`.
macro_rules! raise_multiple {
    // Singular case
    ($result:ident; $raise:expr $(,)?) => {
        $result.or_raise($raise)?
    };

    // Multiple case
    ($($result:ident),+ ; $raise:expr $(,)?) => {{
        $(let $result = $result.or_raise($raise)?;)+
        ($($result,)+)
    }};
}
