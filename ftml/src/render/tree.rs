/*
 * render/tree.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

//! A renderer which outputs a formatted view of the input AST.
//! For debugging or some other trivial renderer need.

use super::Render;
use crate::{PageInfo, Result, SyntaxTree};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TreeRender;

impl Render for TreeRender {
    type Output = String;

    fn render(tree: &SyntaxTree, info: PageInfo) -> Result<String> {
        #[derive(Debug)]
        struct Article<'r, 't, 'i> {
            info: PageInfo<'i>,
            tree: &'r SyntaxTree<'t>,
        }

        let data = Article { info, tree };
        Ok(format!("{:#?}", data))
    }
}
