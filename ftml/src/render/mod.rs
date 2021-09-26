/*
 * render/mod.rs
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

mod prelude {
    pub use super::Render;
    pub use crate::log::prelude::*;
    pub use crate::tree::{AttributeMap, Container, ContainerType, Element, SyntaxTree};
    pub use crate::PageInfo;
}

pub mod debug;
pub mod html;
pub mod json;
pub mod null;
pub mod text;

mod handle;

/// The default format string for `[[date]]`, if none is specified.
pub const DEFAULT_DATETIME_FORMAT: &str = "%B %d, %Y %I:%M:%S %p";

use self::handle::{Handle, ModuleRenderMode};
use crate::log::prelude::*;
use crate::tree::SyntaxTree;
use crate::PageInfo;

/// Abstract trait for any ftml renderer.
///
/// Any structure implementing this trait represents a renderer,
/// with whatever state it needs to perform a rendering of the
/// inputted abstract syntax tree.
pub trait Render {
    /// The type outputted by this renderer.
    ///
    /// Typically this would be a string of some kind,
    /// however more complex renderers may opt to return
    /// types with more information or structure than that,
    /// if they wish.
    type Output;

    /// Render an abstract syntax tree into its output type.
    ///
    /// This is the main method of the trait, causing this
    /// renderer instance to perform whatever operations
    /// it requires to produce the output string.
    fn render(
        &self,
        log: &Logger,
        page_info: &PageInfo,
        tree: &SyntaxTree,
    ) -> Self::Output;
}
