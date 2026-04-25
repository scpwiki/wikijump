/*
 * locales/mod.rs
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

//! This module concerns parsing and using the localization bundle `locales/`.
//!
//! The exposed structures and functions permit easy use of the bundle to
//! perform basic operations.
//!
//! General locale logic should _not_ go in here; considering what it does
//! choose a location like `utils/locale.rs` or the service using the code
//! instead.

#![allow(unused_imports)]

mod arguments;
mod fallback;
mod fluent;

pub use self::{
    arguments::{MessageArguments, MessageValue},
    fallback::iterate_locale_fallbacks,
    fluent::Localizations,
};
