/*
 * parse/paragraph.rs
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

use super::consume::consume;
use super::rule::{Consumption, GenericConsumption};
use super::stack::ParseStack;
use super::token::{ExtractedToken, Token};
use super::ParseException;
use crate::text::FullText;
use crate::tree::Element;

/// Function to iterate over tokens to produce elements in paragraphs.
///
/// Originally in `parse()`, but was moved out to allow paragraph
/// extraction deeper in code, such as in the `try_paragraph`
/// collection helper.
pub fn gather_paragraphs<'l, 'r, 't>(
    log: &'l slog::Logger,
    mut tokens: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> ParseStack<'l, 't> {
    info!(log, "Gathering paragraphs until ending");

    todo!()
}
