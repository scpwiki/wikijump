/*
 * wasm/macros.rs
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

macro_rules! js_to_rust {
    ($js:expr) => {{
        use crate::wasm::error::error_to_js;
        serde_wasm_bindgen::from_value($js).map_err(error_to_js)
    }};
}

macro_rules! rust_to_js {
    ($object:expr) => {{
        use crate::wasm::error::error_to_js;
        serde_wasm_bindgen::to_value(&$object).map_err(error_to_js)
    }};
}
