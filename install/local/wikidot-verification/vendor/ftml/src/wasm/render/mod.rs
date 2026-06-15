/*
 * wasm/render/mod.rs
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

#[cfg(feature = "html")]
mod html;

#[cfg(feature = "html")]
pub use self::html::*;

use super::page_info::PageInfo;
use super::parsing::SyntaxTree;
use super::prelude::*;
use super::settings::WikitextSettings;
use crate::render::Render;
use crate::render::text::TextRender;

// Function exports

#[wasm_bindgen]
pub fn render_text(
    syntax_tree: SyntaxTree,
    page_info: PageInfo,
    settings: WikitextSettings,
) -> String {
    let tree = syntax_tree.get();
    let page_info = page_info.get();
    let settings = settings.get();
    let text = TextRender.render(tree, page_info, settings);

    text
}
