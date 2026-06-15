/*
 * preproc/mod.rs
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

//! This module mimics the Wikidot preprocessor, which replaces certian character sequences to make
//! them look better, or be easier to parse.

pub mod typography;
pub mod whitespace;

#[cfg(test)]
mod test;

use regex::Regex;

/// Helper struct to easily perform string replacements.
#[derive(Debug)]
pub enum Replacer {
    /// Replaces any text matching the "repl" group,
    /// (or the entire regular expression if "repl" does not exist)
    /// with the static string.
    RegexReplace {
        regex: Regex,
        replacement: &'static str,
    },

    /// Takes text matching the regular expression, and replaces the exterior.
    ///
    /// The regular expression must return the content to be preserved in
    /// capture group 1, and surrounds it with the `begin` and `end` strings.
    ///
    /// For instance, say:
    /// * `regex` matched `[% (.+) %]`
    /// * `begin` was `<(`
    /// * `end` was `)>`
    ///
    /// Then input string `[% wikidork %]` would become `<(wikidork)>`.
    RegexSurround {
        regex: Regex,
        begin: &'static str,
        end: &'static str,
    },
}

impl Replacer {
    /// Replaces the text in the manner defined by its enum, using the buffer as a temporary space
    /// to copy to.
    fn replace(&self, text: &mut String, buffer: &mut String) {
        use self::Replacer::*;

        match *self {
            RegexReplace {
                ref regex,
                replacement,
            } => {
                trace!(
                    "Running regex regular expression replacement (pattern {}, replacement {})",
                    regex.as_str(),
                    replacement,
                );

                let mut offset = 0;

                while let Some(capture) = regex.captures_at(text, offset) {
                    let range = {
                        let mtch = capture
                            .name("repl")
                            .unwrap_or_else(|| capture.get(0).unwrap()); // alternative is full match

                        offset = mtch.start() + replacement.len();
                        mtch.range()
                    };

                    text.replace_range(range, replacement);
                }
            }
            RegexSurround {
                ref regex,
                begin,
                end,
            } => {
                trace!(
                    "Running surround regular expression capture replacement (pattern {}, begin {}, end {})",
                    regex.as_str(),
                    begin,
                    end,
                );

                let mut offset = 0;

                while let Some(capture) = regex.captures_at(text, offset) {
                    let mtch = capture
                        .get(1)
                        .expect("Regular expression lacks a content group");

                    let range = {
                        let full_mtch = capture
                            .get(0)
                            .expect("Regular expression lacks a full match");

                        offset = full_mtch.start() + mtch.len() + begin.len() + end.len();
                        full_mtch.range()
                    };

                    buffer.clear();
                    buffer.push_str(begin);
                    buffer.push_str(mtch.as_str());
                    buffer.push_str(end);

                    text.replace_range(range, buffer);
                }
            }
        }
    }
}

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
    info!("Beginning preprocessing of text ({} bytes)", text.len());
    whitespace::substitute(text);
    typography::substitute(text);
    debug!("Finished preprocessing of text ({} bytes)", text.len());
}

#[test]
fn fn_type() {
    type SubstituteFn = fn(&mut String);

    let _: SubstituteFn = whitespace::substitute;
    let _: SubstituteFn = typography::substitute;
}
