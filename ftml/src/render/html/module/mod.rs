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
mod rate;
mod listpages;

mod prelude {
    pub use crate::{Error, Result};
    pub use std::borrow::Cow;
    pub use std::collections::HashMap;
    pub use std::fmt::Write;
    pub use super::Module;
    pub use super::super::HtmlContext;
}

use self::prelude::*;
use self::css::CssModule;
use self::listpages::ListPagesModule;
use self::rate::RateModule;

pub trait Module {
    fn render(
        ctx: &mut HtmlContext,
        arguments: &HashMap<&str, Cow<str>>,
        contents: Option<&str>,
    ) -> Result<()>;
}

pub fn render(
    name: &str,
    ctx: &mut HtmlContext,
    arguments: &HashMap<&str, Cow<str>>,
    contents: Option<&str>,
) -> Result<()> {
    if name.eq_ignore_ascii_case("css") | name.eq_ignore_ascii_case("style") {
        CssModule::render(ctx, arguments, contents)?;
    } else if name.eq_ignore_ascii_case("rate") | name.eq_ignore_ascii_case("rating") {
        RateModule::render(ctx, arguments, contents)?;
    } else if name.eq_ignore_ascii_case("listpages") | name.eq_ignore_ascii_case("list_pages") {
        ListPagesModule::render(ctx, arguments, contents)?;
    } else {
        return Err(Error::Msg(format!("No such module: '{}'", name)));
    }

    Ok(())
}
