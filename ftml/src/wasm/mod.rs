/*
 * wasm/mod.rs
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

#[macro_use]
mod macros;

mod error;
mod log;
mod misc;
mod page_info;
mod parsing;
mod preproc;
mod render;
mod tokenizer;
mod utf16;

mod prelude {
    pub use super::log::LOGGER;
    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen::JsCast;
}

#[cfg(feature = "wasm-log")]
pub use self::log::ConsoleLogger;

pub use self::misc::version;
pub use self::parsing::{parse, ParseOutcome, SyntaxTree};
pub use self::preproc::preprocess;
pub use self::render::{render_html, render_text};
pub use self::tokenizer::{tokenize, Tokenization};
