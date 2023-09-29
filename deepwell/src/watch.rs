/*
 * watch.rs
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

use crate::config::Config;
use anyhow::Result;
use notify::{
    Config as WatcherConfig, Event, EventKind, RecommendedWatcher, RecursiveMode,
    Result as WatcherResult, Watcher,
};
use std::convert::Infallible;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{env, fs};

const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug)]
struct WatchedPaths {
    config_path: PathBuf,
    localization_path: PathBuf,
}

pub fn setup_autorestart(config: &Config) -> Result<RecommendedWatcher> {
    tide::log::info!("Starting watcher for auto-restart on file change");
    let watched_paths = WatchedPaths {
        config_path: fs::canonicalize(&config.raw_toml_path)?,
        localization_path: fs::canonicalize(&config.localization_path)?,
    };

    let mut watcher = RecommendedWatcher::new(
        move |result: WatcherResult<Event>| match result {
            Err(error) => {
                tide::log::error!("Unable to receive filesystem events: {error}");
            }
            Ok(event) => {
                tide::log::debug!(
                    "Received filesystem event ({} paths)",
                    event.paths.len(),
                );

                if event_is_applicable(&watched_paths, event) {
                    restart_self();
                }
            }
        },
        WatcherConfig::default().with_poll_interval(POLL_INTERVAL),
    )?;

    // Add autowatch to configuration file.
    tide::log::debug!("Adding regular watch to {}", config.raw_toml_path.display());
    watcher.watch(&config.raw_toml_path, RecursiveMode::NonRecursive)?;

    // Add autowatch to localization directory.
    // Recursive because it is nested.
    tide::log::debug!(
        "Adding recursive watch to {}",
        config.localization_path.display(),
    );
    watcher.watch(&config.localization_path, RecursiveMode::Recursive)?;

    // Return. Once out of scope, the watcher stops working, so we must preserve it.
    Ok(watcher)
}

fn event_is_applicable(watched_paths: &WatchedPaths, event: Event) -> bool {
    if matches!(
        event.kind,
        EventKind::Access(_) | EventKind::Any | EventKind::Other,
    ) {
        tide::log::debug!("Ignoring access or unknown event");
        return false;
    }

    for path in event.paths {
        if path_is_applicable(watched_paths, &path) {
            return true;
        }
    }

    false
}

fn path_is_applicable(watched_paths: &WatchedPaths, path: &Path) -> bool {
    tide::log::debug!("Checking filesystem event for {}", path.display());

    let path = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(error) => {
            tide::log::error!(
                "Error finding canonical path for event processing: {error}",
            );
            return false;
        }
    };

    if path.starts_with(&watched_paths.config_path) {
        tide::log::info!("DEEPWELL configuration path modified: {}", path.display());
        return true;
    }

    if path.starts_with(&watched_paths.localization_path) {
        tide::log::info!("Localization subpath modified: {}", path.display());
        return true;
    }

    false
}

fn restart_self() -> Infallible {
    tide::log::info!("Restarting server");

    let executable = env::current_exe().expect("Unable to get current executable");
    let arguments = env::args_os().skip(1).collect::<Vec<_>>();

    tide::log::info!(
        "Replacing process with exec: {} {:?}",
        Path::new(&executable).display(),
        arguments,
    );

    let mut command = Command::new(executable);
    command.args(arguments);

    let error = command.exec();
    panic!("Unable to exec(): {error}");
}
