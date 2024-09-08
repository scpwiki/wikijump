/*
 * services/blob/mime.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
//! So we cannot use `once_cell::Lazy` and we can't have it in a coroutine.
//! We don't load the `Magic` instance locally because it's an expensive operation
//! and it would be inefficient to load it for each invocation.
//!
//! Instead we have it in a thread and ferry requests and responses back and forth.

use super::prelude::*;
use filemagic::{FileMagicError, Flags as MagicFlags, Magic};
use std::thread;
use tokio::sync::{mpsc, oneshot};

type RequestPayload = (Vec<u8>, ResponseSender);
type ResponsePayload = StdResult<String, FileMagicError>;

type RequestSender = mpsc::Sender<RequestPayload>;
type RequestReceiver = mpsc::Receiver<RequestPayload>;

type ResponseSender = oneshot::Sender<ResponsePayload>;
type ResponseReceiver = oneshot::Receiver<ResponsePayload>;

#[derive(Debug, Clone)]
pub struct MimeAnalyzer {
    sink: RequestSender,
}

impl MimeAnalyzer {
    /// Starts the MIME analyzer and returns an instance of this struct.
    ///
    /// This launches a new thread to take MIME requests and then returns
    /// a means of communicating with this thread to the caller so calls can be made.
    ///
    /// While technically multiple `MimeAnalyzer` instances could be made, this
    /// is very wasteful; you should only create and use one.
    ///
    /// This object is cheaply cloneable and should be reused instead of
    /// making new instances and starting new threads.
    pub fn spawn() -> Self {
        info!("Starting MIME analyzer worker");
        let (sink, source) = mpsc::channel(64);

        thread::spawn(|| {
            let magic = Self::load_magic().expect("Unable to load magic database");
            Self::main_loop(magic, source);
        });

        MimeAnalyzer { sink }
    }

    /// Loads the libmagic database from file, failing if it was invalid or missing.
    fn load_magic() -> Result<Magic> {
        const MAGIC_FLAGS: MagicFlags = MagicFlags::MIME;
        const MAGIC_PATHS: &[&str] = &[]; // Empty indicates using the default magic database

        info!("Loading magic database data");
        let magic = Magic::open(MAGIC_FLAGS)?;
        magic.load(MAGIC_PATHS)?;
        Ok(magic)
    }

    /// Main loop for the MIME analyzer.
    ///
    /// Runs in a dedicated thread due to borrow checker issues, taking in
    /// requests via a mpsc channel.
    fn main_loop(magic: Magic, mut source: RequestReceiver) {
        while let Some((bytes, sender)) = source.blocking_recv() {
            debug!("Received MIME request ({} bytes)", bytes.len());
            let result = magic.buffer(&bytes);
            sender.send(result).expect("Response channel is closed");
        }

        panic!("MIME magic channel closed (this usually happens when the main application crashes)");
    }

    /// Requests that libmagic analyze the buffer to determine its MIME type.
    ///
    /// Because all requests involve sending an item over the channel,
    /// and then waiting for the response, we need to send both the input
    /// and a oneshot channel to get the response.
    pub async fn get_mime_type(&self, buffer: Vec<u8>) -> Result<String> {
        info!("Sending MIME request ({} bytes)", buffer.len());

        // Channel for getting the result
        let (resp_send, resp_recv): (ResponseSender, ResponseReceiver) =
            oneshot::channel();

        // Send the request
        self.sink
            .send((buffer, resp_send))
            .await
            .expect("MIME channel is closed");

        // Wait for the response
        //
        // Two layers of result for channel failure and MIME request failure
        let resp = resp_recv.await.expect("Response channel is closed");
        let mime = resp?;
        Ok(mime)
    }
}

#[tokio::test]
async fn mime_request() {
    const PNG: &[u8] = b"\x89\x50\x4e\x47\x0d\x0a\x1a\x0a\x00\x00\x00\x0d\x49\x48\x44\x52\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\x04\x73\x42\x49\x54\x08\x08\x08\x08\x7c\x08\x64\x88\x00\x00\x00\x0b\x49\x44\x41\x54\x08\x99\x63\xf8\x0f\x04\x00\x09\xfb\x03\xfd\xe3\x55\xf2\x9c\x00\x00\x00\x00\x49\x45\x4e\x44\xae\x42\x60\x82";
    const TAR_GZIP: &[u8] =
        b"\x1f\x8b\x08\x08\xb1\xb7\x8f\x62\x00\x03\x78\x00\x03\x00\x00\x00\x00";

    let mime = MimeAnalyzer::spawn();

    macro_rules! check {
        ($bytes:expr, $expected:expr $(,)?) => {{
            let future = mime.get_mime_type($bytes.to_vec());
            let actual = future.await.expect("Unable to get MIME type");

            assert_eq!(actual, $expected, "Actual MIME type doesn't match expected");
        }};
    }

    check!(b"", "application/x-empty; charset=binary");
    check!(b"Apple banana", "text/plain; charset=us-ascii");
    check!(PNG, "image/png; charset=binary");
    check!(TAR_GZIP, "application/gzip; charset=binary");
}
