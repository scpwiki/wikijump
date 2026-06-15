/*
 * tree/partial.rs
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

use super::{ListItem, RubyText, Tab, TableCell, TableRow};
use crate::parsing::ParseErrorKind;

/// Part of an element, as returned by a rule.
///
/// These are used by specific rules attempting to
/// build complex or nested structures. From any other
/// context, they are errors are parsing will fail.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PartialElement<'t> {
    /// An item or sub-list within some list.
    ListItem(ListItem<'t>),

    /// A row within some table.
    TableRow(TableRow<'t>),

    /// A cell within some table row.
    TableCell(TableCell<'t>),

    /// A particular tab within a tab view.
    Tab(Tab<'t>),

    /// Text associated with a Ruby annotation.
    ///
    /// Outputs HTML `<rt>`. See also <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby>.
    RubyText(RubyText<'t>),
}

impl PartialElement<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            PartialElement::ListItem(_) => "ListItem",
            PartialElement::TableRow(_) => "TableRow",
            PartialElement::TableCell(_) => "TableCell",
            PartialElement::Tab(_) => "Tab",
            PartialElement::RubyText(_) => "RubyText",
        }
    }

    #[inline]
    pub fn parse_error_kind(&self) -> ParseErrorKind {
        match self {
            PartialElement::ListItem(_) => ParseErrorKind::ListItemOutsideList,
            PartialElement::TableRow(_) => ParseErrorKind::TableRowOutsideTable,
            PartialElement::TableCell(_) => ParseErrorKind::TableCellOutsideTable,
            PartialElement::Tab(_) => ParseErrorKind::TabOutsideTabView,
            PartialElement::RubyText(_) => ParseErrorKind::RubyTextOutsideRuby,
        }
    }

    pub fn to_owned(&self) -> PartialElement<'static> {
        match self {
            PartialElement::ListItem(list_item) => {
                PartialElement::ListItem(list_item.to_owned())
            }
            PartialElement::TableRow(table_row) => {
                PartialElement::TableRow(table_row.to_owned())
            }
            PartialElement::TableCell(table_cell) => {
                PartialElement::TableCell(table_cell.to_owned())
            }
            PartialElement::Tab(tab) => PartialElement::Tab(tab.to_owned()),
            PartialElement::RubyText(text) => PartialElement::RubyText(text.to_owned()),
        }
    }
}

/// A marker enum counterpart to `PartialElement`.
///
/// This is a flag to the parser which designates which
/// partial (if any) the rule is currently looking to accept.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AcceptsPartial {
    #[default]
    None,
    ListItem,
    TableRow,
    TableCell,
    Tab,
    Ruby,
}

impl AcceptsPartial {
    pub fn matches(self, partial: &PartialElement) -> bool {
        matches!(
            (self, partial),
            (AcceptsPartial::ListItem, PartialElement::ListItem(_))
                | (AcceptsPartial::TableRow, PartialElement::TableRow(_))
                | (AcceptsPartial::TableCell, PartialElement::TableCell(_))
                | (AcceptsPartial::Tab, PartialElement::Tab(_))
                | (AcceptsPartial::Ruby, PartialElement::RubyText(_))
        )
    }
}
