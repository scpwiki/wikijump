/*
 * wasm/mod.rs
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

#[macro_use]
mod macros;

mod error;
mod misc;
mod page_info;
mod parsing;
mod preproc;
mod render;
mod settings;
mod tokenizer;
mod utf16;

mod prelude {
    pub use wasm_bindgen::prelude::*;
}

pub use self::misc::version;
pub use self::parsing::{ParseOutcome, SyntaxTree, parse};
pub use self::preproc::preprocess;
pub use self::render::render_text;
pub use self::settings::WikitextSettings;
pub use self::tokenizer::{Tokenization, tokenize};

#[cfg(feature = "html")]
pub use self::render::render_html;
