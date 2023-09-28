/*
 * restart.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

//! Automatically restart server when configuration changes are detected.
//!
//! This is gated behind the `notify` feature, and will restart the entire
//! process when changes to the localization files or the server configuration
//! file are detected.
//!
//! For localization at least, in principle this could load the files in-place,
//! but the server start-up is fast enough, and the borrow checker changes needed
//! would be much more intrusive.
//!
//! This feature is intended for _local development only_, please do not use in production!
//!
//! This feature assumes you are running on a UNIX-like system.

use crate::config::Config;
use notify_debouncer_mini::{new_debouncer, notify::*, DebounceEventResult};

pub fn setup_autorestart(config: &Config) -> Result<()> {
    todo!()
}
