/*
 * preproc/mod.rs
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

mod include;
mod misc;
mod typography;

#[cfg(test)]
mod test;

use crate::Handle;

/// Run the preprocessor on the given wikitext, which is modified in-place.
///
/// The following modifications are performed:
/// * Expand instances of `[[include]]`
/// * Replacing DOS and legacy Mac newlines
/// * Trimming whitespace lines
/// * Concatenating lines that end with backslashes
/// * Convert tabs to four spaces
/// * Compress groups of 3+ newlines into 2 newlines
/// * Wikidot typography transformations
///
/// This call always succeeds. The return value designates where issues occurred
/// to allow programmatic determination of where things were not as expected.
pub fn preprocess(text: &mut String, handle: &dyn Handle) {
    include::substitute(text, handle);
    misc::substitute(text);
    typography::substitute(text);
}

#[test]
fn test_fn() {
    type SubstituteFn = fn(&mut String);
    type SubstituteHandleFn = fn(&mut String, &dyn Handle);

    let _: SubstituteHandleFn = include::substitute;
    let _: SubstituteFn = misc::substitute;
    let _: SubstituteFn = typography::substitute;
}
