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
mod bibliography;
mod code;
mod comment;
mod concat_lines;
mod date;
mod div_prefilter;
mod equation;
mod form;
mod footnote;
mod html;
mod if_tags;
mod iframe;
mod include;
mod link;
mod math;
mod math_inline;
mod module;
mod prefilter;
mod raw;

use crate::{ParseState, Result};
use self::Rule::*;
use self::bibliography::rule_bibliography;
use self::code::rule_code;
use self::comment::rule_comment;
use self::concat_lines::rule_concat_lines;
use self::date::rule_date;
use self::div_prefilter::rule_div_prefilter;
use self::equation::rule_equation;
use self::form::rule_form;
use self::footnote::rule_footnote;
use self::html::rule_html;
use self::if_tags::rule_iftags;
use self::iframe::rule_iframe;
use self::include::rule_include;
use self::link::rule_link;
use self::prefilter::rule_prefilter;
use self::math::rule_math;
use self::math_inline::rule_math_inline;
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
    Link,
    Equation,
    Footnote,
    Bibliography,
    Html,
    DivPrefilter,
    Anchor,
    User,
    Blockquote,
    Heading,
    TableOfContents,
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
            Date => rule_date(state)?,
            Math => rule_math(state)?,
            ConcatLines => rule_concat_lines(state)?,
            Link => rule_link(state)?,
            Equation => rule_equation(state)?,
            Footnote => rule_footnote(state)?,
            Bibliography => rule_bibliography(state)?,
            Html => rule_html(state)?,
            DivPrefilter => rule_div_prefilter(state)?,
            _ => println!("MOCK: rule not implemented yet"),
            /*
             TODO
            Anchor,
            User,
            Blockquote,
            Heading,
            TableOfContents,
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
            MathInline => rule_math_inline(state)?,
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
pub const RULES: [Rule; 60] = [
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
    Link,
    Equation,
    Footnote,
    Bibliography,
    Html,
    DivPrefilter,
    Anchor,
    User,
    Blockquote,
    Heading,
    TableOfContents,
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
    let _: ApplyFn = rule_date;
    let _: ApplyFn = rule_math;
    let _: ApplyFn = rule_math_inline;
    let _: ApplyFn = rule_concat_lines;
    let _: ApplyFn = rule_link;
    let _: ApplyFn = rule_footnote;
    let _: ApplyFn = rule_bibliography;
    let _: ApplyFn = rule_div_prefilter;

    // TODO for all the other functions
}
