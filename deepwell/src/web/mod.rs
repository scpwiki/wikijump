/*
 * web/mod.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

mod category;
mod connection_type;
mod provided_value;
mod reference;
mod unwrap;
mod user_details;

pub mod ratelimit;
pub mod utils;

pub use self::category::*;
pub use self::connection_type::ConnectionType;
pub use self::provided_value::ProvidedValue;
pub use self::reference::Reference;
pub use self::unwrap::HttpUnwrap;
pub use self::user_details::{UserDetails, UserDetailsQuery};
