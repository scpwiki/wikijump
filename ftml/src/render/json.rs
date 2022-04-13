/*
 * render/json.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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

//! A simple renderer that outputs the `SyntaxTree` as JSON.
//!
//! This implementation of `Render` will produce the same JSON
//! output as is used in the AST tests at `src/test.rs`.

use super::prelude::*;

#[derive(Debug)]
pub struct JsonRender {
    /// Whether to use the human-readable JSON formatter or the minified formatter.
    pub pretty: bool,
}

impl JsonRender {
    #[inline]
    pub fn pretty() -> Self {
        JsonRender { pretty: true }
    }

    #[inline]
    pub fn compact() -> Self {
        JsonRender { pretty: false }
    }
}

impl Render for JsonRender {
    type Output = String;

    fn render(
        &self,
        syntax_tree: &SyntaxTree,
        page_info: &PageInfo,
        settings: &WikitextSettings,
    ) -> String {
        info!(
            "Running JSON logger on syntax tree (pretty {})",
            self.pretty,
        );

        // Get the JSON serializer
        let writer = if self.pretty {
            serde_json::to_string_pretty
        } else {
            serde_json::to_string
        };

        // Wrapper struct to provide both page info and the AST in the JSON.
        #[derive(Serialize, Debug)]
        #[serde(rename_all = "kebab-case")]
        struct JsonWrapper<'a> {
            settings: &'a WikitextSettings,
            page_info: &'a PageInfo<'a>,
            syntax_tree: &'a SyntaxTree<'a>,
        }

        let output = JsonWrapper {
            settings,
            page_info,
            syntax_tree,
        };

        writer(&output).expect("Unable to serialize JSON")
    }
}
