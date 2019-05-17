/*
 * wdhtmlserv/server.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

use crate::{Request, Response};
use json;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::process;
use wikidot_html::prelude::*;

const BUFFER_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Server {
    socket: UnixDatagram,
}

impl Server {
    pub fn new(path: &Path) -> Self {
        let socket = match UnixDatagram::bind(path) {
            Ok(sock) => sock,
            Err(err) => {
                error!("Couldn't bind to '{}': {}", path.display(), err);
                process::exit(1);
            }
        };

        Server { socket }
    }

    pub fn process(&self, _request: Request) -> Result<()> {
        unimplemented!()
    }

    pub fn main_loop(&mut self) -> ! {
        let mut buffer = [0; BUFFER_SIZE];

        loop {
            match self.socket.recv_from(&mut buffer[..]) {
                Ok((size, addr)) => {
                    let slice = &buffer[..size];
                    let request = match json::from_slice(slice) {
                        Ok(req) => req,
                        Err(err) => {
                            warn!("Error deserializing request from JSON: {}", err);
                            continue;
                        }
                    };

                    let response = match self.process(request) {
                        Ok(resp) => Response::success(resp),
                        Err(err) => {
                            warn!("Error processing request: {}", err);
                            continue;
                        }
                    };

                    let data = match json::to_vec(&response) {
                        Ok(data) => data,
                        Err(err) => {
                            error!("Error serializing response to JSON: {}", err);
                            continue;
                        }
                    };

                    let path = addr.as_pathname()
                        .expect("Received datagram wasn't from a path");
                    match self.socket.send_to(&data, path) {
                        Ok(bytes) if bytes == data.len() => (),
                        Ok(bytes) => warn!(
                            "Socket send only sent {} of {} bytes of response",
                            bytes,
                            data.len()
                        ),
                        Err(err) => warn!("Error sending response over socket: {}", err),
                    }
                }
                Err(err) => error!("Error receiving request from socket: {}", err),
            }
        }
    }
}
