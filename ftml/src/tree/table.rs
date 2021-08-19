/*
 * tree/table.rs
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

use super::clone::elements_to_owned;
use super::{AttributeMap, Element};
use std::num::NonZeroU32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Table<'t> {
    pub rows: Vec<TableRow<'t>>,
    pub attributes: AttributeMap<'t>,
}

impl Table<'_> {
    pub fn to_owned(&self) -> Table<'static> {
        Table {
            rows: self.rows.iter().map(|row| row.to_owned()).collect(),
            attributes: self.attributes.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TableRow<'t> {
    pub cells: Vec<TableCell<'t>>,
    pub attributes: AttributeMap<'t>,
}

impl TableRow<'_> {
    pub fn to_owned(&self) -> TableRow<'static> {
        TableRow {
            cells: self.cells.iter().map(|cell| cell.to_owned()).collect(),
            attributes: self.attributes.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TableCell<'t> {
    pub contents: Vec<Element<'t>>,
    pub header: bool,
    pub column_span: NonZeroU32,
    pub attributes: AttributeMap<'t>,
}

impl TableCell<'_> {
    pub fn to_owned(&self) -> TableCell<'static> {
        TableCell {
            contents: elements_to_owned(&self.contents),
            header: self.header,
            column_span: self.column_span,
            attributes: self.attributes.to_owned(),
        }
    }
}
