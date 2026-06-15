/*
 * parsing/strip.rs
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

use crate::tree::Element;

pub fn strip_newlines(elements: &mut Vec<Element>) {
    // Remove leading line breaks
    while let Some(element) = elements.first() {
        if !matches!(element, Element::LineBreak | Element::LineBreaks(_)) {
            break;
        }

        elements.remove(0);
    }

    // Remove trailing line breaks
    while let Some(element) = elements.last() {
        if !matches!(element, Element::LineBreak | Element::LineBreaks(_)) {
            break;
        }

        elements.pop();
    }
}

pub fn strip_whitespace(elements: &mut Vec<Element>) {
    // Remove leading whitespace
    while let Some(element) = elements.first() {
        if !element.is_whitespace() {
            break;
        }

        elements.remove(0);
    }

    // Remove trailing whitespace
    while let Some(element) = elements.last() {
        if !element.is_whitespace() {
            break;
        }

        elements.pop();
    }
}
