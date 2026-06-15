/*
 * parsing/exception.rs
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

use super::{ExtractedToken, Token, rule::Rule};
use crate::utf16::Utf16IndexMap;
use serde::{Serializer, ser::SerializeTuple};
use std::borrow::Cow;
use std::ops::Range;
use strum_macros::IntoStaticStr;

/// An issue that occurred during parsing.
///
/// These refer to circumstances where a rule was attempted, but did not
/// succeed due to an issue with the syntax.
///
/// However, as outlined by the crate's philosophy, no parsing issue is fatal.
/// Instead a fallback rules is applied and parsing continues.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ParseError {
    token: Token,
    rule: Cow<'static, str>,
    #[serde(serialize_with = "serialize_span")]
    span: Range<usize>,
    kind: ParseErrorKind,
}

impl ParseError {
    #[inline]
    pub fn new(kind: ParseErrorKind, rule: Rule, current: &ExtractedToken) -> Self {
        let token = current.token;
        let span = Range::clone(&current.span);
        let rule = cow!(rule.name());

        ParseError {
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
    pub fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    #[must_use]
    pub fn to_utf16_indices(&self, map: &Utf16IndexMap) -> Self {
        // Copy fields
        let ParseError {
            token,
            rule,
            span,
            kind,
        } = self.clone();

        // Map indices to UTF-16
        let start = map.get_index(span.start);
        let end = map.get_index(span.end);
        let span = start..end;

        // Output new error
        ParseError {
            token,
            rule,
            span,
            kind,
        }
    }
}

#[derive(Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ParseErrorKind {
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

    /// Ruby text block appears outside of a ruby annotation block.
    RubyTextOutsideRuby,

    /// Bibliography contains an element other than a definition list.
    BibliographyContainsNonDefinitionList,

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

impl ParseErrorKind {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}

/// Helper function to serialize spans as a 2-tuple.
fn serialize_span<S>(span: &Range<usize>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tuple = serializer.serialize_tuple(2)?;
    tuple.serialize_element(&span.start)?;
    tuple.serialize_element(&span.end)?;
    tuple.end()
}
