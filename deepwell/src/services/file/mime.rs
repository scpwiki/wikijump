/*
 * services/file/mime.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
use async_std::task;
use filemagic::{Flags as MagicFlags, Magic};
use std::{process, thread};
use void::{ResultVoidErrExt, Void};

fn run_magic_thread() -> Result<Void> {
    const MAGIC_FLAGS: MagicFlags = MagicFlags::MIME;
    const MAGIC_PATHS: &[&str] = &[]; // Empty indicates using the default magic database

    tide::log::info!("Loading magic database constant");
    let magic = Magic::open(MAGIC_FLAGS)?;
    magic.load(MAGIC_PATHS)?;

    loop {
        let buffer: &[u8] = &[];
        let _ = magic.buffer(buffer);
    }
}

/// Starts a thread which contains the `Magic` instance.
///
/// Because it is a binding to a C library, it cannot be shared among threads.
/// So we cannot use `lazy_static` and we can't have it in a coroutine.
/// We don't load the `Magic` instance locally because it's an expensive operation
/// and it would be inefficient to load it for each invocation.
///
/// Instead we have it in a thread and ferry requests and responses back and forth.
pub fn spawn_magic_thread() {
    thread::spawn(|| {
        // Since this is an infinite loop, no success case can return.
        // Only the initialization can fail, individual requests just pass back the result.
        let error = run_magic_thread().void_unwrap_err();
        tide::log::error!("Failed to spawn magic thread: {error}");
        process::exit(1);
    });
}

#[inline]
pub fn mime_type(buffer: &[u8]) -> Result<String> {
    todo!()
}
