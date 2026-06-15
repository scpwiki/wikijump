/*
 * wasm/settings.rs
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

use super::prelude::*;
use crate::layout::Layout as RustLayout;
use crate::settings::{
    WikitextMode as RustWikitextMode, WikitextSettings as RustWikitextSettings,
};
use std::sync::Arc;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WikitextSettings {
    inner: Arc<RustWikitextSettings>,
}

#[wasm_bindgen]
impl WikitextSettings {
    #[inline]
    pub(crate) fn get(&self) -> &RustWikitextSettings {
        &self.inner
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> WikitextSettings {
        WikitextSettings {
            inner: Arc::clone(&self.inner),
        }
    }

    #[wasm_bindgen(constructor)]
    pub fn new(settings: JsValue) -> Result<WikitextSettings, JsValue> {
        Ok(WikitextSettings {
            inner: Arc::new(js_to_rust!(settings)?),
        })
    }

    #[wasm_bindgen]
    pub fn from_mode(mode: String, layout: String) -> Result<WikitextSettings, JsValue> {
        let rust_mode = match mode.as_str() {
            "page" => RustWikitextMode::Page,
            "draft" => RustWikitextMode::Draft,
            "forum-post" => RustWikitextMode::ForumPost,
            "direct-message" => RustWikitextMode::DirectMessage,
            "list" => RustWikitextMode::List,
            _ => return Err(JsValue::from_str("Unknown mode")),
        };

        let rust_layout = match layout.as_str() {
            "wikidot" => RustLayout::Wikidot,
            "wikijump" => RustLayout::Wikijump,
            _ => return Err(JsValue::from_str("Unknown layout")),
        };

        Ok(WikitextSettings {
            inner: Arc::new(RustWikitextSettings::from_mode(rust_mode, rust_layout)),
        })
    }
}
