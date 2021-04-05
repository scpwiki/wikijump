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

use self::log::{CONSOLE_LOGGER, NULL_LOGGER};
use crate::info;
use ouroboros::self_referencing;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

mod log;

pub use self::log::ConsoleLogger;

fn get_logger(should_log: bool) -> &'static slog::Logger {
    if should_log {
        &*CONSOLE_LOGGER
    } else {
        &*NULL_LOGGER
    }
}

#[wasm_bindgen]
pub fn version() -> String {
    info::VERSION.clone()
}

#[wasm_bindgen]
pub fn preprocess(mut text: String, should_log: bool) -> String {
    let log = get_logger(should_log);

    crate::preprocess(log, &mut text);
    text
}
