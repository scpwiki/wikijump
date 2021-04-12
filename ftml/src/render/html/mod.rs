/*
 * render/html/mod.rs
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

// TODO! remove when this is implemented
#![allow(dead_code)]

#[cfg(test)]
mod test;

#[macro_use]
mod macros;

mod builder;
mod context;
mod escape;
mod meta;
mod output;
mod render;

pub use self::meta::{HtmlMeta, HtmlMetaType};
pub use self::output::HtmlOutput;

#[cfg(test)]
use super::prelude;

use crate::render::Render;
use crate::tree::SyntaxTree;

#[derive(Debug)]
pub struct HtmlRender;

impl Render for HtmlRender {
    type Output = HtmlOutput;

    fn render(&self, _log: &slog::Logger, _tree: &SyntaxTree) -> HtmlOutput {
        todo!()
    }
}
