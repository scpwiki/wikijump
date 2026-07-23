/*
 * config/runtime_action.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

//! Perform a "runtime action" instead of normal server execution.
//!
//! This is useful in contexts such as CI, where we want DEEPWELL to
//! not run as a daemon, but instead perform a special action or check,
//! as if motivated by a script.

use super::{Config, SetupConfig};
use crate::services::corpus_render_finalizer::{
    CorpusRenderFinalizerService, RenderFinalizerSettings,
};
use crate::services::corpus_render_inventory::{
    CorpusRenderInventoryService, RenderInventorySettings,
};
use crate::services::render::{
    RenderReplayService, RenderReplaySettings, run_worker_action,
};
use crate::{api, database};
use std::path::PathBuf;
use std::{env, process};

pub async fn run_runtime_action() {
    // Get action name, if specified.
    // Otherwise return and perform normal execution.
    let Ok(action_name) = env::var("DEEPWELL_RUNTIME_ACTION") else {
        return;
    };

    // Run appropriate runtime action.
    let return_code = match action_name.as_str() {
        "config" | "validate-config" => validate_config(),
        "seeder" | "run-seeder" => run_seeder().await,
        "render-finalize" => run_render_finalize().await,
        "render-inventory" => run_render_inventory().await,
        "render-replay" => run_render_replay().await,
        "render-replay-worker" => run_worker_action(),
        _ => {
            eprintln!("Unknown runtime action: {action_name}");
            process::exit(1);
        }
    };

    // Exit, don't perform normal server execution.
    process::exit(return_code);
}

fn validate_config() -> i32 {
    println!("Running action: Validate configuration");

    let mut return_code = 0;
    for value in env::args_os().skip(1) {
        let path = PathBuf::from(value);
        print!("Checking {}... ", path.display());

        match Config::load(path) {
            Ok(_) => println!("success"),
            Err(error) => {
                println!("error:");
                eprintln!("{error:?}");
                return_code += 1;
            }
        }
    }

    println!("All config files have been checked, {return_code} failed");
    return_code
}

async fn run_seeder() -> i32 {
    println!("Running action: Database seeder");

    let SetupConfig { secrets, config } = SetupConfig::load_only();
    let app_state = match api::build_server_state_without_workers(config, secrets).await {
        Ok(app_state) => app_state,
        Err(error) => {
            println!("error:");
            eprintln!("{error:?}");
            return 1;
        }
    };

    match database::seed(&app_state).await {
        Ok(()) => {
            println!("success");
            0
        }
        Err(error) => {
            println!("error:");
            eprintln!("{error:?}");
            1
        }
    }
}

async fn run_render_finalize() -> i32 {
    let settings = match RenderFinalizerSettings::from_env() {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("{error:?}");
            return 1;
        }
    };

    let SetupConfig { secrets, config } = SetupConfig::load_only();
    let app_state = match api::build_server_state_without_workers(config, secrets).await {
        Ok(app_state) => app_state,
        Err(error) => {
            eprintln!("{error:?}");
            return 1;
        }
    };

    match CorpusRenderFinalizerService::run(&app_state, settings).await {
        Ok(summary) => match serde_json::to_string(&summary) {
            Ok(json) => {
                println!("{json}");
                0
            }
            Err(error) => {
                eprintln!("{error:?}");
                1
            }
        },
        Err(error) => {
            eprintln!("{error:?}");
            1
        }
    }
}

async fn run_render_inventory() -> i32 {
    let settings = match RenderInventorySettings::from_env() {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("{error:?}");
            return 1;
        }
    };

    let SetupConfig { secrets, config } = SetupConfig::load_only();
    let app_state = match api::build_server_state_without_workers(config, secrets).await {
        Ok(app_state) => app_state,
        Err(error) => {
            eprintln!("{error:?}");
            return 1;
        }
    };

    match CorpusRenderInventoryService::run(&app_state, settings).await {
        Ok(summary) => match serde_json::to_string(&summary) {
            Ok(json) => {
                println!("{json}");
                if summary.passed() { 0 } else { 2 }
            }
            Err(error) => {
                eprintln!("{error:?}");
                1
            }
        },
        Err(error) => {
            eprintln!("{error:?}");
            1
        }
    }
}

async fn run_render_replay() -> i32 {
    let settings = match RenderReplaySettings::from_env() {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("{error:?}");
            return 1;
        }
    };

    let SetupConfig { secrets, config } = SetupConfig::load_only();
    let app_state = match api::build_server_state_without_workers(config, secrets).await {
        Ok(app_state) => app_state,
        Err(error) => {
            eprintln!("{error:?}");
            return 1;
        }
    };

    match RenderReplayService::run(&app_state, settings).await {
        Ok(summary) => match serde_json::to_string(&summary) {
            Ok(json) => {
                println!("{json}");
                if summary.gate_passed() { 0 } else { 1 }
            }
            Err(error) => {
                eprintln!("{error:?}");
                1
            }
        },
        Err(error) => {
            eprintln!("{error:?}");
            1
        }
    }
}
