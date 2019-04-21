/*
 * filter/mod.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
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

// TODO, in this order probably?
//
// * Includes
// * Prefilter stuff. copy
// * Concat lines
// + Convert quote blocks to [[quote]] ... [[/quote]]
// * Typography: https://github.com/Nu-SCPTheme/wikidot/blob/master/lib/Text_Wiki/Text/Wiki/Parse/Default/Typography.php

mod quote_block;

/// Transform the text in preparation for parsing.
///
/// Performs the following modifications:
/// * (TODO)
/// * Converts quote blocks to nested [[quote]] tags.
pub fn prefilter(text: &mut String) {
    quote_block::substitute(text);
}
