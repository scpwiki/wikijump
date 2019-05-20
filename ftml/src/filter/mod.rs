/*
 * filter/mod.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

mod blockquote;
mod include;
mod misc;
mod typography;

#[cfg(test)]
mod test;

pub use self::include::Includer;
pub use self::include::{NotFoundIncluder, NullIncluder};

use crate::Result;

/// Transform the text in preparation for parsing.
///
/// Performs the following modifications:
/// * Insert [[include]] directives
/// * Replacing DOS and legacy Mac newlines
/// * Trimming whitespace lines
/// * Concatenating lines that end with backslashes
/// * Convert tabs to four spaces
/// * Compress groups of 3+ newlines into 2 newlines
/// * Converts quote blocks to nested [[quote]] tags
/// * Perform typography modifications
pub fn prefilter(text: &mut String, includer: &Includer) -> Result<()> {
    include::substitute(text, includer)?;
    misc::substitute(text)?;
    blockquote::substitute(text)?;
    typography::substitute(text)?;

    Ok(())
}

/// Transform the output text following rendering.
///
/// Currently performs no action.
/// Performs the following modifications:
#[inline]
pub fn postfilter(_text: &mut String) -> Result<()> {
    Ok(())
}

#[test]
fn test_fn() {
    type SubstituteFn = fn(&mut String) -> Result<()>;

    // include::substitute() does not match as it requires an Includer
    let _: SubstituteFn = misc::substitute;
    let _: SubstituteFn = blockquote::substitute;
    let _: SubstituteFn = typography::substitute;
}
