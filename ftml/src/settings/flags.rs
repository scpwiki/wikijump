/*
 * settings/flags.rs
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

#[bitflags]
#[repr(u16)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ParserRender {
    // Whether certain syntactical constructs are allowed
    DisableInclude,
    DisableModule,
    DisableTableOfContents,
    DisableButton,

    // Whether real IDs should be used, or randomly generated
    HeadingRandomId,
    FootnoteRandomId,
    BibliographyRandomId,
    MathRandomId,

    // Whether local paths are permitted
    DisableLocalPaths,
}
