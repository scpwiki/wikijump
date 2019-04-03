/*
 * parse/rules/mod.rs
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

// Rule implementations
mod code;
mod comment;
mod form;
mod if_tags;
mod iframe;
mod include;
mod module;
mod prefilter;
mod raw;

use crate::{ParseState, Result};
use self::Rule::*;
use self::code::rule_code;
use self::comment::rule_comment;
use self::form::rule_form;
use self::if_tags::rule_iftags;
use self::iframe::rule_iframe;
use self::include::rule_include;
use self::prefilter::rule_prefilter;
use self::module::rule_module;
use self::raw::rule_raw;

#[derive(Debug, Copy, Clone)]
pub enum Rule {
    Include,
    Prefilter,
    Code,
    Form,
    Raw,
    Module,
    IfTags,
    Comment,
    IFrame,
    Date,
    Math,
    ConcatLines,
    FreeLink,
    EquationReference,
    Footnote,
    FootnoteItem,
    FootnoteBlock,
    BibItem,
    Bibliography,
    BibCite,
    Html,
    DivPrefilter,
    Anchor,
    User,
    Blockquote,
    Heading,
    Toc,
    Horiz,
    Separator,
    ClearFloat,
    Break,
    Span,
    Size,
    Div,
    DivAlign,
    Collapsible,
    TabView,
    Note,
    Gallery,
    List,
    DefList,
    Table,
    TableAdv,
    Image,
    Embed,
    Social,
    File,
    Center,
    Newline,
    Paragraph,
    Url,
    Email,
    MathInline,
    Interwiki,
    Colortext,
    Strong,
    Emphasis,
    Underline,
    Strikethrough,
    Teletype,
    Superscript,
    Subscript,
    Typography,
    Tighten,
}

impl Rule {
    pub fn apply(self, state: &mut ParseState) -> Result<()> {
        match self {
            Include => rule_include(state)?,
            Prefilter => rule_prefilter(state)?,
            Code => rule_code(state)?,
            Form => rule_form(state)?,
            Raw => rule_raw(state)?,
            Module => rule_module(state)?,
            IfTags => rule_iftags(state)?,
            Comment => rule_comment(state)?,
            IFrame => rule_iframe(state)?,
            _ => println!("MOCK: unknown rule"),
            /*
             TODO
            Date,
            Math,
            ConcatLines,
            FreeLink,
            EquationReference,
            Footnote,
            FootnoteItem,
            FootnoteBlock,
            BibItem,
            Bibliography,
            BibCite,
            Html,
            DivPrefilter,
            Anchor,
            User,
            Blockquote,
            Heading,
            Toc,
            Horiz,
            Separator,
            ClearFloat,
            Break,
            Span,
            Size,
            Div,
            DivAlign,
            Collapsible,
            TabView,
            Note,
            Gallery,
            List,
            DefList,
            Table,
            TableAdv,
            Image,
            Embed,
            Social,
            File,
            Center,
            Newline,
            Paragraph,
            Url,
            Email,
            MathInline,
            Interwiki,
            Colortext,
            Strong,
            Emphasis,
            Underline,
            Strikethrough,
            Teletype,
            Superscript,
            Subscript,
            Typography,
            Tighten,
            */
        }

        Ok(())
    }
}

// Copied from Wikidot Text_Wiki source
// For maximum backwards-compatibility, leave as-is
pub const RULES: [Rule; 64] = [
    Include,
    Prefilter,
    Code,
    Form,
    Raw,
    Module,
    IfTags,
    Comment,
    IFrame,
    Date,
    Math,
    ConcatLines,
    FreeLink,
    EquationReference,
    Footnote,
    FootnoteItem,
    FootnoteBlock,
    BibItem,
    Bibliography,
    BibCite,
    Html,
    DivPrefilter,
    Anchor,
    User,
    Blockquote,
    Heading,
    Toc,
    Horiz,
    Separator,
    ClearFloat,
    Break,
    Span,
    Size,
    Div,
    DivAlign,
    Collapsible,
    TabView,
    Note,
    Gallery,
    List,
    DefList,
    Table,
    TableAdv,
    Image,
    Embed,
    Social,
    File,
    Center,
    Newline,
    Paragraph,
    Url,
    Email,
    MathInline,
    Interwiki,
    Colortext,
    Strong,
    Emphasis,
    Underline,
    Strikethrough,
    Teletype,
    Superscript,
    Subscript,
    Typography,
    Tighten,
];

#[test]
fn test_variants() {
    let mut state = ParseState::new(String::new());
    for rule in &RULES[..] {
        rule.apply(&mut state).unwrap();
    }

    assert_eq!("\n\n", state.text());
    assert_eq!(0, state.tokens().len());
}

#[test]
fn test_fn_types() {
    type ApplyFn = fn(&mut ParseState) -> Result<()>;

    let _: ApplyFn = rule_include;
    let _: ApplyFn = rule_prefilter;
    let _: ApplyFn = rule_code;
    let _: ApplyFn = rule_form;
    let _: ApplyFn = rule_raw;
    let _: ApplyFn = rule_module;
    let _: ApplyFn = rule_iftags;
    let _: ApplyFn = rule_comment;
    let _: ApplyFn = rule_iframe;

    // TODO for all the other functions
}
