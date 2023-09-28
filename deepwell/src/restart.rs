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
//! Note the [security implications of `current_exe()`](https://doc.rust-lang.org/std/env/fn.current_exe.html#security).
//!
//! This feature assumes you are running on a UNIX-like system.
//! If on Linux, then inotify will be used.

use crate::api::ApiServerState;
use crate::config::Config;
use notify_debouncer_mini::{
    new_debouncer, notify::*, DebounceEventResult, DebouncedEvent, Debouncer,
};
use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use void::Void;

const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

pub fn setup_autorestart(state: &ApiServerState) -> Result<Debouncer<impl Watcher>> {
    let raw_toml_path = &state.config.raw_toml_path;
    let localization_path = &state.config.localization_path;
    let state = Arc::clone(&state);

    let mut debouncer = new_debouncer(
        DEBOUNCE_DURATION,
        move |result: DebounceEventResult| match result {
            Err(error) => {
                tide::log::error!("Unable to receive filesystem events: {error}");
            }
            Ok(events) => {
                tide::log::debug!("Received {} filesystem events", events.len());

                let should_restart = events
                    .iter()
                    .any(|event| event_is_applicable(&state.config, event));

                if should_restart {
                    restart_self();
                }
            }
        },
    )?;

    // Add autowatch to configuration file.
    let watcher = debouncer.watcher();
    watcher.watch(raw_toml_path, RecursiveMode::NonRecursive)?;

    // Add autowatch to localization directory.
    // Recursive because it is nested.
    watcher.watch(localization_path, RecursiveMode::Recursive)?;

    // Return. Once out of scope, the watcher stops working.
    Ok(debouncer)
}

fn event_is_applicable(
    config: &Config,
    DebouncedEvent { path, .. }: &DebouncedEvent,
) -> bool {
    tide::log::debug!("Checking filesystem event for {}", path.display());

    if path.starts_with(&config.raw_toml_path) {
        tide::log::info!("DEEPWELL configuration path modified: {}", path.display());
        return true;
    }

    if path.starts_with(&config.localization_path) {
        tide::log::info!("Localization subpath modified: {}", path.display());
        return true;
    }

    false
}

fn restart_self() -> Void {
    tide::log::info!("Restarting server");

    // TODO
    todo!()
}
