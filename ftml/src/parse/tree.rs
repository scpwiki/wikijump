/*
 * parse/tree.rs
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

// FIXME to prevent compile spam
#![allow(dead_code)]

use crate::enums::{Alignment, ListStyle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree<'a> {
    paragraphs: Vec<Paragraph<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Paragraph<'a> {
    Align {
        alignment: Alignment,
    },
    Button {
        /*
         https://www.wikidot.com/doc-wiki-syntax:buttons
         btype: ButtonType,
         style: String,
         */
    },
    Center {
        contents: Box<Word<'a>>,
    },
    ClearFloat {
        direction: Option<Alignment>,
    },
    CodeBlock {
        language: Option<&'a str>,
        contents: Box<Paragraph<'a>>,
    },
    Div {
        class: Option<&'a str>,
        style: Option<&'a str>,
    },
    FootnoteBlock,
    Form {
        contents: &'a str, // actually YAML...
    },
    Gallery,
    Heading {
        contents: Box<Word<'a>>,
    },
    HorizontalLine,
    Html {
        contents: &'a str,
    },
    Iframe {
        url: &'a str,
        args: Option<&'a str>,
    },
    IfTags {
        required: Vec<&'a str>,
        prohibited: Vec<&'a str>,
        contents: Box<Paragraph<'a>>,
    },
    List {
        style: ListStyle,
        items: Vec<Word<'a>>,
    },
    Math {
        label: Option<&'a str>,
        id: Option<&'a str>,
        latex_env: Option<&'a str>,
        expr: &'a str,
    },
    Module {
        name: &'a str,
        contents: Option<Box<Paragraph<'a>>>,
    },
    Note {
        contents: Box<Paragraph<'a>>,
    },
    Table {
        rows: Vec<TableRow<'a>>,
    },
    TabView {
        class: Option<&'a str>,
        tabs: Vec<Paragraph<'a>>,
    },
    TableOfContents {
        // TODO: http://community.wikidot.com/help:toc
    },
    Text {
        contents: Word<'a>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Word<'a> {
    Anchor {
        name: &'a str,
    },
    Bold {
        contents: Box<Word<'a>>,
    },
    Color {
        color: &'a str,
    },
    Date {
        timestamp: i64,
        format: Option<&'a str>,
    },
    Email {
        contents: &'a str,
    },
    EquationReference {
        name: &'a str,
    },
    File {
        filename: &'a str,
    },
    Footnote {
        contents: Box<Paragraph<'a>>,
    },
    Image {
        // See https://www.wikidot.com/doc-wiki-syntax:images
        filename: &'a str,
        link: Option<(&'a str, bool)>,
        alt: Option<&'a str>,
        title: Option<&'a str>,
        width: Option<&'a str>,
        height: Option<&'a str>,
        style: Option<&'a str>,
        class: Option<&'a str>,
        size: Option<&'a str>,
    },
    Italics {
        contents: Box<Word<'a>>,
    },
    Link {
        page: &'a str,
        anchor: Option<&'a str>,
        text: Option<&'a str>,
    },
    Math {
        expr: &'a str,
    },
    Monospace {
        contents: Box<Word<'a>>,
    },
    Raw {
        contents: &'a str,
    },
    Size {
        size: &'a str,
        contents: Box<Word<'a>>,
    },
    Span {
        id: Option<&'a str>,
        class: Option<&'a str>,
        style: Option<&'a str>,
        contents: Box<Word<'a>>,
    },
    Strikethrough {
        contents: Box<Word<'a>>,
    },
    Subscript {
        contents: Box<Word<'a>>,
    },
    Superscript {
        contents: Box<Word<'a>>,
    },
    Text {
        contents: &'a str,
    },
    Underline {
        contents: Box<Word<'a>>,
    },
    Url {
        contents: &'a str,
    },
    User {
        username: &'a str,
        show_picture: bool,
    },
    Words {
        words: Vec<Word<'a>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow<'a> {
    columns: Vec<Word<'a>>,
    title: bool,
}
