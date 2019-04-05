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

use crate::enums::{Alignment, ListStyle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree {
    paragraphs: Vec<Paragraph>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Paragraph {
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
    ClearFloat {
        direction: Option<Alignment>,
    },
    CodeBlock {
        language: Option<String>,
        contents: Box<Paragraph>,
    },
    Div {
        class: Option<String>,
        style: Option<String>,
    },
    FootnoteBlock,
    Form {
        contents: String, // actually YAML...
    },
    Gallery,
    Heading {
        contents: Box<Word>,
    },
    HorizontalLine,
    Html {
        contents: String,
    },
    Iframe {
        url: String,
        args: Option<String>,
    },
    IfTags {
        required: Vec<String>,
        prohibited: Vec<String>,
        contents: Box<Paragraph>,
    },
    Math {
        label: Option<String>,
        id: Option<String>,
        latex_env: Option<String>,
        expr: String,
    },
    Module {
        name: String,
        contents: Option<Box<Paragraph>>,
    },
    Note {
        contents: Box<Paragraph>,
    },
    Table {
        rows: Vec<TableRow>,
    },
    TabView {
        class: Option<String>,
        tabs: Vec<Paragraph>,
    },
    TableOfContents {
        // TODO: http://community.wikidot.com/help:toc
    },
    Text {
        contents: Word,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Word {
    Anchor {
        name: String,
    },
    Bold {
        contents: Box<Word>,
    },
    Center {
        contents: Box<Word>,
    },
    Color {
        color: String,
    },
    Date {
        timestamp: i64,
        format: Option<String>,
    },
    Email {
        contents: String,
    },
    EquationReference {
        name: String,
    },
    File {
        filename: String,
    },
    Footnote {
        contents: Box<Paragraph>,
    },
    Image {
        // See https://www.wikidot.com/doc-wiki-syntax:images
        filename: String,
        link: Option<(String, bool)>,
        alt: Option<String>,
        title: Option<String>,
        width: Option<String>,
        height: Option<String>,
        style: Option<String>,
        class: Option<String>,
        size: Option<String>,
    },
    Italics {
        contents: Box<Word>,
    },
    Link {
        page: String,
        anchor: Option<String>,
        text: Option<String>,
    },
    List {
        style: ListStyle,
        items: Vec<Word>,
    },
    Math {
        expr: String,
    },
    Monospace {
        contents: Box<Word>,
    },
    Raw {
        contents: String,
    },
    Size {
        size: String,
        contents: Box<Word>,
    },
    Span {
        id: Option<String>,
        class: Option<String>,
        style: Option<String>,
        contents: Box<Word>,
    },
    Strikethrough {
        contents: Box<Word>,
    },
    Subscript {
        contents: Box<Word>,
    },
    Superscript {
        contents: Box<Word>,
    },
    Text {
        contents: String,
    },
    Underline {
        contents: Box<Word>,
    },
    Url {
        contents: String,
    },
    User {
        username: String,
        show_picture: bool,
    },
    Words {
        words: Vec<Word>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    columns: Vec<Word>,
    title: bool,
}
