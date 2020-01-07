/*
 * server.rs
 *
 * ftml-rpc - RPC server to convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

use crate::rpc::*;
use futures::future::{self, Ready};
use std::net::SocketAddr;
use std::time::SystemTime;
use tarpc::context::Context;

const PROTOCOL_VERSION: &str = "0";

#[derive(Debug, Copy, Clone)]
pub struct Server(SocketAddr);

impl Server {
    #[inline]
    pub fn new(addr: SocketAddr) -> Self {
        Server(addr)
    }
}

// Misc

impl Protocol for Server {
    type ProtocolFut = Ready<&'static str>;

    #[inline]
    fn protocol(self, _: Context) -> Self::ProtocolFut {
        info!("Method call: protocol");

        future::ready(PROTOCOL_VERSION)
    }
}

impl Ping for Server {
    type PingFut = Ready<&'static str>;

    #[inline]
    fn ping(self, _: Context) -> Self::PingFut {
        info!("Method call: ping");

        future::ready("pong!")
    }
}

impl Time for Server {
    type TimeFut = Ready<f64>;

    fn time(self, _: Context) -> Self::TimeFut {
        info!("Method call: time");

        let now = SystemTime::now();
        let unix_time = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time before epoch")
            .as_secs_f64();

        future::ready(unix_time)
    }
}
