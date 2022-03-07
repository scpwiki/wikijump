/*
 * preproc/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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

mod typography;
mod whitespace;

#[cfg(test)]
mod test;

/// Run the preprocessor on the given wikitext, which is modified in-place.
///
/// The following modifications are performed:
/// * Replacing DOS and legacy Mac newlines
/// * Trimming whitespace lines
/// * Concatenating lines that end with backslashes
/// * Convert tabs to four spaces
/// * Wikidot typography transformations
///
/// This call always succeeds. The return value designates where issues occurred
/// to allow programmatic determination of where things were not as expected.
pub fn preprocess(text: &mut String) {
    whitespace::substitute(text);
    typography::substitute(text);
    info!("Finished preprocessing of text");
}

#[test]
fn fn_type() {
    type SubstituteFn = fn(&mut String);

    let _: SubstituteFn = whitespace::substitute;
    let _: SubstituteFn = typography::substitute;
}
