/*
 * render/html/meta.rs
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

use super::escape as html;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HtmlMetaType {
    Name,
    HttpEquiv,
    Property,
}

impl HtmlMetaType {
    pub fn tag_name(self) -> &'static str {
        use self::HtmlMetaType::*;

        match self {
            Name => "name",
            HttpEquiv => "http-equiv",
            Property => "property",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HtmlMeta {
    pub tag_type: HtmlMetaType,
    pub name: String,
    pub value: String,
}

impl HtmlMeta {
    pub fn render(&self, buffer: &mut String) {
        str_write!(buffer, "<meta {}=\"", self.tag_type.tag_name());
        html::escape(buffer, &self.name);
        buffer.push_str("\" content=\"");
        html::escape(buffer, &self.value);
        buffer.push_str("\" />");
    }
}
