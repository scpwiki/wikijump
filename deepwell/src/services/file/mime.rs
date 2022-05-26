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

//! Evaluates MIME types using libmagic.
//!
//! Because it is a binding to a C library, it cannot be shared among threads.
//! So we cannot use `lazy_static` and we can't have it in a coroutine.
//! We don't load the `Magic` instance locally because it's an expensive operation
//! and it would be inefficient to load it for each invocation.
//!
//! Instead we have it in a thread and ferry requests and responses back and forth.

use super::prelude::*;
use async_std::task;
use crossfire::mpsc;
use filemagic::{FileMagicError, Flags as MagicFlags, Magic};
use std::{process, thread};
use void::{ResultVoidErrExt, Void};

type ResponsePayload = StdResult<String, FileMagicError>;
type ResponseSender = mpsc::TxBlocking<ResponsePayload, mpsc::SharedSenderBRecvF>;

type RequestPayload = (Vec<u8>, ResponseSender);
type RequestSender = mpsc::TxFuture<RequestPayload, mpsc::SharedSenderFRecvB>;
type RequestReceiver = mpsc::RxBlocking<RequestPayload, mpsc::SharedSenderFRecvB>;

lazy_static! {
    static ref QUEUE: (RequestSender, RequestReceiver) =
        mpsc::bounded_tx_future_rx_blocking(64);
}

macro_rules! sink {
    () => {
        QUEUE.0
    };
}

macro_rules! source {
    () => {
        QUEUE.1
    };
}

fn main_loop() -> Result<Void> {
    const MAGIC_FLAGS: MagicFlags = MagicFlags::MIME;
    const MAGIC_PATHS: &[&str] = &[]; // Empty indicates using the default magic database

    tide::log::info!("Loading magic database constant");
    let magic = Magic::open(MAGIC_FLAGS)?;
    magic.load(MAGIC_PATHS)?;

    loop {
        tide::log::debug!("Waiting for next MIME request");

        let (bytes, sender) = source!().recv().expect("MIME channel has disconnected");
        let result = magic.buffer(&bytes);
        sender
            .send(result)
            .expect("MIME response channel has disconnected");
    }
}

/// Starts the thread containing the `Magic` instance.
pub fn spawn_magic_thread() {
    thread::spawn(|| {
        // Since this is an infinite loop, no success case can return.
        // Only the initialization can fail, individual requests just pass back the result.
        let error = main_loop().void_unwrap_err();
        tide::log::error!("Failed to spawn magic thread: {error}");
        process::exit(1);
    });
}

/// Requests that libmagic analyze the buffer to determine its MIME type.
///
/// Because all requests involve sending an item over the channel,
/// and then waiting for the response, we need to send both the input
/// and a oneshot channel to get the response.
pub async fn mime_type(buffer: Vec<u8>) -> Result<String> {
    // One-shot equivalent channel
    let (resp_send, resp_recv) = mpsc::bounded_tx_blocking_rx_future(1);

    // Send request
    sink!()
        .send((buffer, resp_send))
        .await
        .expect("MIME channel has disconnected");

    // Wait for response
    let result = resp_recv
        .recv()
        .await
        .expect("MIME response channel has disconnected");

    let mime = result?;
    Ok(mime)
}

#[test]
fn mime_request() {
    const PNG: &[u8] = b"\x89\x50\x4e\x47\x0d\x0a\x1a\x0a\x00\x00\x00\x0d\x49\x48\x44\x52\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\x04\x73\x42\x49\x54\x08\x08\x08\x08\x7c\x08\x64\x88\x00\x00\x00\x0b\x49\x44\x41\x54\x08\x99\x63\xf8\x0f\x04\x00\x09\xfb\x03\xfd\xe3\x55\xf2\x9c\x00\x00\x00\x00\x49\x45\x4e\x44\xae\x42\x60\x82";
    const TAR: &[u8] =
        b"\x1f\x8b\x08\x08\xb1\xb7\x8f\x62\x00\x03\x78\x00\x03\x00\x00\x00\x00";

    let sender = spawn_magic_thread();

    macro_rules! check {
        ($bytes:expr, $expected:expr $(,)?) => {{
            let future = mime_type(&sender, $bytes.to_vec());
            let actual = task::block_on(future).expect("Unable to get MIME type");

            assert_eq!(actual, $expected, "Actual MIME type doesn't match expected");
        }};
    }

    check!(b"", "application/x-empty; charset=binary");
    check!(b"Apple banana", "text/plain; charset=us-ascii");
    check!(PNG, "image/png; charset=binary");
    check!(TAR, "application/gzip; charset=binary");
}
