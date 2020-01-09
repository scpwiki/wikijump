/*
 * render/html/meta.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019-2020 Ammon Smith
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

use super::builder as html;
use super::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HtmlMeta {
    pub tag_type: HtmlMetaType,
    pub name: String,
    pub value: String,
}

impl HtmlMeta {
    pub fn render(&self, buffer: &mut String) -> Result<()> {
        write!(buffer, "<meta {}=\"", self.tag_type.tag_name())?;
        html::escape(buffer, &self.name);
        buffer.push_str("\" content=\"");
        html::escape(buffer, &self.value);
        buffer.push_str("\" />");
        Ok(())
    }
}
