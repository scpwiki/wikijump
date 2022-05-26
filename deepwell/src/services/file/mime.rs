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
use async_std::channel::{self, Receiver, Sender};
use async_std::task;
use filemagic::{FileMagicError, Flags as MagicFlags, Magic};
use std::{process, thread};
use void::{ResultVoidErrExt, Void};

pub type MagicResponsePayload = StdResult<String, FileMagicError>;
pub type MagicResponseSender = Sender<MagicResponsePayload>;
pub type MagicResponseReceiver = Receiver<MagicResponsePayload>;

pub type MagicPayload = (Vec<u8>, MagicResponseSender);
pub type MagicSender = Sender<MagicPayload>;
pub type MagicReceiver = Receiver<MagicPayload>;

fn run_magic_thread(receiver: MagicReceiver) -> Result<Void> {
    const MAGIC_FLAGS: MagicFlags = MagicFlags::MIME;
    const MAGIC_PATHS: &[&str] = &[]; // Empty indicates using the default magic database

    tide::log::info!("Loading magic database constant");
    let magic = Magic::open(MAGIC_FLAGS)?;
    magic.load(MAGIC_PATHS)?;

    loop {
        tide::log::debug!("Waiting for next MIME request");

        let (bytes, sender) =
            task::block_on(receiver.recv()).expect("Unable to receive MIME request");

        let result = magic.buffer(&bytes);

        task::block_on(sender.send(result)) //
            .expect("Unable to send MIME response");
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
    let (send, recv) = channel::unbounded();

    thread::spawn(move || {
        // Since this is an infinite loop, no success case can return.
        // Only the initialization can fail, individual requests just pass back the result.
        let error = run_magic_thread(recv).void_unwrap_err();
        tide::log::error!("Failed to spawn magic thread: {error}");
        process::exit(1);
    });
}

/// Requests that libmagic analyze the buffer to determine its MIME type.
///
/// Because all requests involve sending an item over the channel,
/// and then waiting for the response, we need to send both the input
/// and a oneshot channel to get the response. However `async_std`
/// doesn't have a separate oneshot channel, so we're using a bounded
/// one instead.
pub async fn mime_type(sender: &MagicSender, buffer: Vec<u8>) -> Result<String> {
    let (resp_send, resp_recv) = channel::bounded(1);

    // Send request
    sender //
        .send((buffer, resp_send))
        .await
        .expect("Unable to send to MIME channel");

    // Wait for response
    let result = resp_recv
        .recv()
        .await
        .expect("Unable to receive from MIME channel");

    let mime = result?;
    Ok(mime)
}
