/*
 * macros.rs
 *
 * ftml - Library to parse Wikidot text
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
///
/// # See also
/// * [`str_write!`](macro.str_write.html)
macro_rules! str_writeln {
    ($dest:expr, $($arg:tt)*) => {{
        use std::fmt::Write;
        writeln!($dest, $($arg)*).expect("Writing to string failed");
    }};
}

#[test]
fn macros() {
    use crate::tree::Element;
    use std::borrow::Cow;

    let cow_value = cow!("alpha");
    assert_eq!(cow_value, Cow::Borrowed("alpha"));
    assert_eq!(&cow_value, "alpha");

    let text_element = text!("beta");
    assert_eq!(text_element, Element::Text(Cow::Borrowed("beta")));

    let mut string = String::new();
    str_write!(&mut string, "foo");
    str_writeln!(&mut string, "bar");
    assert_eq!(&string, "foobar\n");
}
