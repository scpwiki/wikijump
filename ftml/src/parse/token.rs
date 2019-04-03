/*
 * parse/token.rs
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

//! Tokens are the Wikidot parser's way of storing complex tree information
//! in what is basically a multiple-pass find-and-replace system. In this
//! implementation, tokens are inserted as two null bytes surrounding an index,
//! which denotes a token to be placed there during rendering.

#[derive(Debug, Clone)]
pub enum Token {
    CodeBlock {
        args: Option<String>,
        contents: String,
        end: String,
    },
    Date {
        timestamp: i64,
        args: Option<String>,
    },
    Form {
        contents: String,
    },
    Iframe {
        url: String,
        args: Option<String>,
    },
    Include {
        page: String,
        args: Option<String>,
    },
    Link {
        page: String,
        anchor: Option<String>,
        text: Option<String>,
    },
    Math {
        label: Option<String>,
        args: Option<String>,
        expr: String,
        end: String,
    },
    MathInline {
        expr: String,
    },
    Module {
        name: String,
        args: Option<String>,
        contents: Option<String>,
    },
    Raw {
        contents: String,
    },
}
