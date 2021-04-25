/*
 * ffi/log.rs
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

use crate::log::prelude::*;

#[cfg(feature = "has-log")]
pub fn get_logger() -> Logger {
    use slog::Drain;
    use std::io;
    use std::sync::Mutex;

    lazy_static! {
        static ref LOGGER: Logger = {
            let drain = slog_bunyan::with_name("ftml", io::stdout())
                .set_newlines(true)
                .set_pretty(false)
                .set_flush(false)
                .build();

            Logger::root(Mutex::new(drain).fuse(), slog_o!())
        };
    }

    Logger::clone(&*LOGGER)
}

#[cfg(not(feature = "has-log"))]
#[inline]
pub fn get_logger() -> Logger {
    Logger
}
