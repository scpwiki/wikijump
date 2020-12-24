/*
 * parse/rule/impls/block/body.rs
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

use crate::parse::token::ExtractedToken;

/// Specifying the manner that this block wants a body.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BodyKind {
    /// This block wants a body composed of elements.
    /// It specifies rather these internals are to be
    /// parsed as paragraphs or solely inline elements.
    ///
    /// Examples: `[[div]]` (true), `[[span]]` (false)
    Elements { paragraphs: bool },

    /// This block wants a text body.
    /// The contents do not want to be seen as tokens,
    /// and it will simply consume all contents until
    /// the ending is found.
    ///
    /// Examples: `[[module CSS]]`, `[[code]]`
    Text,

    /// This block doesn't accept a body.
    /// It is simply a freestanding element.
    ///
    /// Examples: `[[module Rate]]`
    None,
}

/// The result of retrieving a block's body.
///
/// See also `BodyKind`.
#[derive(Debug, Clone, PartialEq)]
pub enum Body<'r, 't> {
    Elements(&'r [ExtractedToken<'t>]),
    Text(&'t str),
    None,
}
