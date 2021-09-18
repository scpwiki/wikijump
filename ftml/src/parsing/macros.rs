/*
 * parsing/macros.rs
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

/// Creates a `ParseResult::Ok` with the given fields.
///
/// There are two variants, for if there are exceptions or if there are not.
macro_rules! ok {
    // Derive paragraph safety
    // Must return Elements instead of T
    ($item:expr $(,)?) => {
        ok!($item, Vec::new())
    };
    ($item:expr, $exceptions:expr $(,)?) => {{
        use crate::parsing::ParseSuccess;
        use crate::tree::PartialElements;

        let item: PartialElements = $item.into();
        let paragraph_safe = item.paragraph_safe();

        Ok(ParseSuccess::new(item, $exceptions, paragraph_safe))

    }};
    // Specify paragraph safety
    ($paragraph_safe:expr; $item:expr $(,)?) => {
        ok!($paragraph_safe; $item, Vec::new())
    };
    ($paragraph_safe:expr; $item:expr, $exceptions:expr $(,)?) => {{
        use crate::parsing::ParseSuccess;

        Ok(ParseSuccess::new($item.into(), $exceptions, $paragraph_safe))
    }};
}

/// Convert a collection of `PartialElements` into `Elements`.
macro_rules! try_from_partials {
    ($parser:expr, $partials:expr) => {{
        use crate::parsing::ParseWarningKind;
        use crate::tree::Elements;
        use std::convert::TryInto;

        let elements: Result<Elements, ParseWarningKind> = $partials.try_into();
        elements.map_err(|warn_kind| $parser.make_warn(warn_kind))?
    }};
}
