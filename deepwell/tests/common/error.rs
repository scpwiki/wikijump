/*
 * tests/common/error.rs
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

//! Functions and macros for processing (expected) error values in tests.

#![allow(unused_macros)]

use deepwell::error::prelude::*;
use exn::{Exn, Frame};

#[allow(dead_code)]
/// Extract the deepwell error from the `Exn<Error>` which matches a condition.
///
/// This walks the error tree until it finds the first `Error` type which
/// matches the `ErrorType`-based condition passed in.
pub fn extract_error<F>(exn_error: &Exn<Error>, condition: F) -> Option<&Error>
where
    F: Fn(&ErrorType) -> bool + Copy,
{
    fn walk<F>(frame: &Frame, condition: F) -> Option<&Error>
    where
        F: Fn(&ErrorType) -> bool + Copy,
    {
        match frame.error().downcast_ref::<Error>() {
            Some(found) if condition(&found.error_type) => Some(found),
            _ => frame
                .children()
                .iter()
                .find_map(|frame| walk(frame, condition)),
        }
    }

    walk(exn_error.frame(), condition)
}

/// Extract the deepwell error from the `Exn<Error>` which matches an error type pattern.
///
/// This is a wrapper macro for `extract_error()` which allows you to pass in a pattern
/// to be evaluated in `matches!`.
macro_rules! extract_error {
    ($exn_error:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {
        crate::common::extract_error(
            &$exn_error,
            |etype| matches!(etype, $pattern $(if $guard)?),
        )
    };
}

/// Asserts that there exists an error type matching the given pattern within the `Exn<Error>`.
macro_rules! assert_contains_error {
    ($exn_error:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {{
        let exn_error = &$exn_error;
        let extracted = extract_error!(exn_error, $pattern $(if $guard)?);
        assert!(
            extracted.is_some(),
            "Cannot find error within trace matching '{}':\n{:?}",
            stringify!($pattern),
            exn_error,
        );
    }};
}

/// Asserts that there are no errors within the `Exn<Error>` matching the given pattern.
macro_rules! assert_no_error {
    ($exn_error:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {{
        let exn_error = &$exn_error;
        let extracted = extract_error!(exn_error, $pattern $(if $guard)?);
        assert!(
            extracted.is_none(),
            "Found error within trace matching '{}': {:?}\n{:?}",
            stringify!($pattern),
            extracted.unwrap(), // known to exist at this point
            exn_error,
        );
    }};
}
