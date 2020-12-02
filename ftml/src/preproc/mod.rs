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

mod typography;
mod whitespace;

#[cfg(test)]
mod test;

/// Run the preprocessor on the given wikitext, which is modified in-place.
///
/// The following modifications are performed:
/// * Expand instances of `[[include]]`
/// * Replacing DOS and legacy Mac newlines
/// * Trimming whitespace lines
/// * Concatenating lines that end with backslashes
/// * Convert tabs to four spaces
/// * Wikidot typography transformations
///
/// This call always succeeds. The return value designates where issues occurred
/// to allow programmatic determination of where things were not as expected.
pub fn preprocess(log: &slog::Logger, text: &mut String) {
    let log = &log.new(slog_o!(
        "filename" => slog_filename!(),
        "lineno" => slog_lineno!(),
        "function" => "preprocess",
        "text" => str!(text),
    ));

    whitespace::substitute(log, text);
    typography::substitute(log, text);

    info!(log, "Finished preprocessing of text"; "text" => &*text);
}

#[test]
fn fn_type() {
    type SubstituteFn = fn(&slog::Logger, &mut String);

    let _: SubstituteFn = whitespace::substitute;
    let _: SubstituteFn = typography::substitute;
}
