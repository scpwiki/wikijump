/*
 * render/html/module/css.rs
 *
 * ftml - Convert Wikidot code to HTML
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

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct CssModule;

impl Module for CssModule {
    fn render(
        ctx: &mut HtmlContext,
        _arguments: &HashMap<&str, Cow<str>>,
        contents: Option<&str>,
    ) -> Result<()> {
        match contents {
            Some(style) => ctx.add_style(style),
            None => return Err(Error::StaticMsg("No style contents in CSS module")),
        }

        Ok(())
    }
}
