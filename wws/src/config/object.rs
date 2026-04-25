/*
 * config/object.rs
 *
 * Wilson's Web Server - Serves a zoo of user-generated content
 * Copyright (C) 2019-2026 Wikijump Team
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

use std::{net::SocketAddr, path::PathBuf};

/// The runtime configuration structure for the web server.
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether to enable tracing and colored backtrace.
    pub enable_trace: bool,

    /// Whether to crash the process if DEEPWELL isn't available at start.
    /// This can cause issues locally, since wws may rebuild before
    /// deepwell does, and then it finds deepwell isn't ready yet.
    pub enable_deepwell_check: bool,

    /// The PID file (if any) to write to on boot.
    pub pid_file: Option<PathBuf>,

    /// The address the server will be hosted on.
    pub address: SocketAddr,
}
