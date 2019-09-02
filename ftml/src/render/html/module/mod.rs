/*
 * render/html/module/mod.rs
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

mod css;
mod listpages;
mod rate;

mod prelude {
    pub use super::super::HtmlContext;
    pub use super::Module;
    pub use crate::{Error, Result};
    pub use std::borrow::Cow;
    pub use std::collections::HashMap;
    pub use std::fmt::Write;
}

use self::css::CssModule;
use self::listpages::ListPagesModule;
use self::prelude::*;
use self::rate::RateModule;

pub type ModuleRenderFn = fn(
    ctx: &mut HtmlContext,
    arguments: &HashMap<&str, Cow<str>>,
    contents: Option<&str>,
) -> Result<()>;

const MODULE_LIST: [(&[&str], ModuleRenderFn); 3] = [
    (&["css", "style"], CssModule::render),
    (&["rate", "rating"], RateModule::render),
    (&["listpages", "list_pags"], ListPagesModule::render),
];

pub trait Module {
    fn render(
        ctx: &mut HtmlContext,
        arguments: &HashMap<&str, Cow<str>>,
        contents: Option<&str>,
    ) -> Result<()>;
}

pub fn render(
    module_name: &str,
    ctx: &mut HtmlContext,
    arguments: &HashMap<&str, Cow<str>>,
    contents: Option<&str>,
) -> Result<()> {
    for (names, module) in &MODULE_LIST[..] {
        for name in names.iter() {
            if module_name.eq_ignore_ascii_case(name) {
                module(ctx, arguments, contents)?;
                return Ok(());
            }
        }
    }

    Err(Error::Msg(format!("No such module: '{}'", module_name)))
}
