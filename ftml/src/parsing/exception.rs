/*
 * parsing/exception.rs
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

use super::{rule::Rule, ExtractedToken, Token};
use crate::utf16::Utf16IndexMap;
use std::borrow::Cow;
use std::ops::Range;
use strum_macros::IntoStaticStr;

/// Exceptions that occurred during parsing
///
/// This is distinct from `ParseWarning` in that it is
/// an internal structure meant to catch exceptional
/// outputs.
///
/// These are primarily parser warnings, but are not necessarily such.
/// For instance, CSS styles are not present in the syntax tree
/// like regular elements, and instead must be bubbled up
/// to the top level.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ParseException<'t> {
    Warning(ParseWarning),
    Style(Cow<'t, str>),
}

/// An issue that occurred during parsing.
///
/// These refer to circumstances where a rule was attempted, but did not
/// succeed due to an issue with the syntax.
///
/// However, as outlined by the crate's philosophy, no parsing issue is fatal.
/// Instead a fallback rules is applied and parsing continues.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ParseWarning {
    token: Token,
    rule: Cow<'static, str>,
    span: Range<usize>,
    kind: ParseWarningKind,
}

impl ParseWarning {
    #[inline]
    pub fn new(kind: ParseWarningKind, rule: Rule, current: &ExtractedToken) -> Self {
        let token = current.token;
        let span = Range::clone(&current.span);
        let rule = cow!(rule.name());

        ParseWarning {
            token,
            rule,
            span,
            kind,
        }
    }

    #[inline]
    pub fn token(&self) -> Token {
        self.token
    }

    #[inline]
    pub fn rule(&self) -> &str {
        &self.rule
    }

    #[inline]
    pub fn span(&self) -> Range<usize> {
        Range::clone(&self.span)
    }

    #[inline]
    pub fn kind(&self) -> ParseWarningKind {
        self.kind
    }

    #[inline]
    #[cfg(feature = "ffi")]
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn rule_raw(&self) -> &Cow<'static, str> {
        &self.rule
    }

    #[must_use]
    pub fn to_utf16_indices(&self, map: &Utf16IndexMap) -> Self {
        // Copy fields
        let ParseWarning {
            token,
            rule,
            span,
            kind,
        } = self.clone();

        // Map indices to UTF-16
        let start = map.get_index(span.start);
        let end = map.get_index(span.end);
        let span = start..end;

        // Output new warning
        ParseWarning {
            token,
            rule,
            span,
            kind,
        }
    }
}

#[derive(Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ParseWarningKind {
    /// The self-enforced recursion limit has been passed, giving up.
    RecursionDepthExceeded,

    /// Attempting to process this rule failed because the end of input was reached.
    EndOfInput,

    /// No rules match for these tokens, returning as plain text.
    NoRulesMatch,

    /// Attempting to match this rule failed, falling back to try an alternate.
    RuleFailed,

    /// This syntax is not supported when parsing in the current mode.
    NotSupportedMode,

    /// Attempting to match this rule failed, it must be on the start of a new line.
    NotStartOfLine,

    /// This include block was malformed, and thus not substituted.
    InvalidInclude,

    /// This list has no elements in it.
    ListEmpty,

    /// This list has elements other than items in it.
    ListContainsNonItem,

    /// This list item is not within a list.
    ListItemOutsideList,

    /// This list tries to nest too deeply.
    ListDepthExceeded,

    /// This table has elements other than rows in it.
    TableContainsNonRow,

    /// This table row has elements other than cells in it.
    TableRowContainsNonCell,

    /// This table row appears outside of a table.
    TableRowOutsideTable,

    /// This table cell appears outside of a table row.
    TableCellOutsideTable,

    /// This tabview has no elements in it.
    TabViewEmpty,

    /// This tabview has elements other than tabs in it.
    TabViewContainsNonTab,

    /// There is a tab outside of a tabview.
    TabOutsideTabView,

    /// Footnotes are not permitted from inside footnotes.
    FootnotesNested,

    /// This native blockquote tries to nest too deeply.
    BlockquoteDepthExceeded,

    /// There is no rule for the block name specified.
    NoSuchBlock,

    /// This block does not allow star (`*`) invocation.
    BlockDisallowsStar,

    /// This block does not allow score (`_`) invocation.
    BlockDisallowsScore,

    /// This block does not specify a name.
    BlockMissingName,

    /// This block does not have close brackets when required.
    BlockMissingCloseBrackets,

    /// Encountered malformed arguments when parsing the block.
    BlockMalformedArguments,

    /// Some required arguments where missing when parsing the block.
    BlockMissingArguments,

    /// This block expected to end its body here.
    BlockExpectedEnd,

    /// An end block was found, but of the incorrect type.
    BlockEndMismatch,

    /// No embed with this name exists.
    NoSuchEmbed,

    /// This no rule for the module name specified.
    NoSuchModule,

    /// This module does not specify a name.
    ModuleMissingName,

    /// The given page to be included does not exist.
    NoSuchPage,

    /// The given variable was not found, and thus not substituted.
    NoSuchVariable,

    /// The URL passed here was invalid.
    InvalidUrl,
}

impl ParseWarningKind {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}

#[cfg(feature = "log")]
impl slog::Value for ParseWarningKind {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}

#[test]
fn log() {
    let log = crate::build_logger();

    info!(
        &log,
        "Received parse warning";
        "warning" => ParseWarningKind::NoRulesMatch,
    );
}
