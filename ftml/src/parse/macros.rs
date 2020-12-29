/*
 * parse/macros.rs
 *
 * ftml - Library to parse Wikidot text
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

/// Creates a `ParseResult::Ok` with the given fields.
///
/// There are two variants, for if there are exceptions or if there are not.
macro_rules! ok {
    ($item:expr, $remaining:expr,) => {
        ok!($item, $remaining, Vec::new())
    };
    ($item:expr, $remaining:expr) => {
        ok!($item, $remaining, Vec::new())
    };
    ($item:expr, $remaining:expr, $exceptions:expr,) => {
        ok!($item, $remaining, $exceptions)
    };
    ($item:expr, $remaining:expr, $exceptions:expr) => {
        Ok(ParseSuccess {
            item: $item,
            remaining: $remaining,
            exceptions: $exceptions,
        })
    };
}

/// Unwraps a `ParseResult`, and then moving the pointer back for a `try_collect` call.
///
/// The macro will call `try_consume!`, then run `last_before_slice` to get the previous token.
///
/// This is necessary because the `try_collect` functions require the first token to be the opener,
/// and the following to be its contents.
macro_rules! try_consume_last {
    ($remaining:expr, $result:expr,) => {
        try_consume_last!($remaining, $result)
    };

    ($remaining:expr, $result:expr) => {{
        let ParseSuccess {
            item,
            remaining: new_remaining,
            exceptions,
        } = $result?;

        let extracted = last_before_slice($remaining, new_remaining);

        (item, extracted, new_remaining, exceptions)
    }};
}
