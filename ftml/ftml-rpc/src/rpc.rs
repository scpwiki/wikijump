/*
 * rpc.rs
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

use crate::Result;
use ftml::html::HtmlOutput;
use serde_json::{Error as JsonError, Value};

// Misc

#[tarpc::service]
pub trait Protocol {
    async fn protocol() -> &'static str;
}

#[tarpc::service]
pub trait Ping {
    async fn ping() -> &'static str;
}

#[tarpc::service]
pub trait Time {
    async fn time() -> f64;
}

// Core

#[tarpc::service]
pub trait Prefilter {
    async fn prefilter(input: String) -> Result<String>;
}

#[tarpc::service]
pub trait Parse {
    async fn parse(input: String) -> Result<Value>;
}

#[tarpc::service]
pub trait Render {
    async fn render(input: String) -> Result<HtmlOutput>;
}
