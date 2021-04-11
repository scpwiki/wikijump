/*
 * wasm/log/mod.rs
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

use cfg_if::cfg_if;

lazy_static! {
    pub static ref NULL_LOGGER: slog::Logger = {
        slog::Logger::root(slog::Discard, o!()) //
    };
}

cfg_if! {
    if #[cfg(feature = "wasm-log")] {
        mod console;
        mod context;

        pub use self::console::ConsoleLogger;

        lazy_static! {
            pub static ref CONSOLE_LOGGER: slog::Logger = {
                use slog::Drain;

                slog::Logger::root(ConsoleLogger.fuse(), o!())
            };
        }

        pub fn get_logger(should_log: bool) -> &'static slog::Logger {
            if should_log {
                &*CONSOLE_LOGGER
            } else {
                &*NULL_LOGGER
            }
        }
    } else {
        #[inline]
        pub fn get_logger(_should_log: bool) -> &'static slog::Logger {
            &*NULL_LOGGER
        }
    }
}
