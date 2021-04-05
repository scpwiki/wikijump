/*
 * wasm/log.rs
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

use slog::Drain;
use void::Void;
use wasm_bindgen::JsValue;
use web_sys::console;

lazy_static! {
    pub static ref NULL_LOGGER: slog::Logger = {
        slog::Logger::root(slog::Discard, o!()) //
    };
    pub static ref CONSOLE_LOGGER: slog::Logger = {
        slog::Logger::root(ConsoleLogger.fuse(), o!()) //
    };
}

type ConsoleLogFn = fn(&JsValue);

#[derive(Debug)]
pub struct ConsoleLogger;

impl slog::Drain for ConsoleLogger {
    type Ok = ();
    type Err = Void;

    fn log(
        &self,
        record: &slog::Record,
        _values: &slog::OwnedKVList,
    ) -> Result<(), Void> {
        let console_log_fn = get_console_fn(record.level());

        let message = record.msg().to_string();

        // TODO actually serialize values, don't just drop them
        // you'll need to use debug_2, log_2, etc variants to pass (string, object)

        console_log_fn(&JsValue::from_str(&message));
        Ok(())
    }
}

fn get_console_fn(level: slog::Level) -> ConsoleLogFn {
    use slog::Level::*;

    match level {
        Trace => console::debug_1,
        Debug => console::log_1,
        Info => console::info_1,
        Warning => console::warn_1,
        Error => console::error_1,
        Critical => console::error_1,
    }
}
