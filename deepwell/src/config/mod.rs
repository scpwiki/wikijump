/*
 * config/mod.rs
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

mod args;
mod object;
mod secrets;

pub use self::object::Config;
pub use self::secrets::Secrets;

#[derive(Debug, Clone)]
pub struct SetupConfig {
    pub secrets: Secrets,
    pub config: Config,
}

impl SetupConfig {
    pub fn load() -> Self {
        let secrets = Secrets::load();
        let config = Config::parse_args();

        SetupConfig { secrets, config }
    }
}
