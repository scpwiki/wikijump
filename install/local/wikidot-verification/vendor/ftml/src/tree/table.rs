/*
 * tree/table.rs
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

use super::clone::elements_to_owned;
use super::{Alignment, AttributeMap, Element};
use std::num::NonZeroU32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Table<'t> {
    #[serde(rename = "type")]
    pub table_type: TableType,
    pub attributes: AttributeMap<'t>,
    pub rows: Vec<TableRow<'t>>,
}

impl Table<'_> {
    pub fn to_owned(&self) -> Table<'static> {
        Table {
            table_type: self.table_type,
            attributes: self.attributes.to_owned(),
            rows: self.rows.iter().map(|row| row.to_owned()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TableType {
    Simple,
    Advanced,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TableRow<'t> {
    pub attributes: AttributeMap<'t>,
    pub cells: Vec<TableCell<'t>>,
}

impl TableRow<'_> {
    pub fn to_owned(&self) -> TableRow<'static> {
        TableRow {
            attributes: self.attributes.to_owned(),
            cells: self.cells.iter().map(|cell| cell.to_owned()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TableCell<'t> {
    pub header: bool,
    pub column_span: NonZeroU32,
    pub align: Option<Alignment>,
    pub attributes: AttributeMap<'t>,
    pub elements: Vec<Element<'t>>,
}

impl TableCell<'_> {
    pub fn to_owned(&self) -> TableCell<'static> {
        TableCell {
            header: self.header,
            column_span: self.column_span,
            align: self.align,
            attributes: self.attributes.to_owned(),
            elements: elements_to_owned(&self.elements),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TableItem<'t> {
    Row(TableRow<'t>),
    Cell(TableCell<'t>),
}

impl TableItem<'_> {
    pub fn to_owned(&self) -> TableItem<'static> {
        match self {
            TableItem::Row(row) => TableItem::Row(row.to_owned()),
            TableItem::Cell(cell) => TableItem::Cell(cell.to_owned()),
        }
    }
}
