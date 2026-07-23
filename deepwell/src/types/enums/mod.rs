/*
 * types/enums/mod.rs
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

//! Enums used across multiple services that are stored as database TEXT values.

#[macro_use]
mod macros;

mod alias_type;
mod connection_type;
mod file_revision;
mod license;
mod message_recipient_type;
mod page_lock_type;
mod page_revision;
mod permission;
mod relation;
mod text_block_type;
mod user_type;

pub use self::alias_type::AliasType;
pub use self::connection_type::ConnectionType;
pub use self::file_revision::FileRevisionType;
pub use self::license::License;
pub use self::message_recipient_type::MessageRecipientType;
pub use self::page_lock_type::PageLockType;
pub use self::page_revision::{PageRevisionChange, PageRevisionType};
pub use self::permission::{Action, Resource};
pub use self::relation::{RelationObjectType, RelationType};
pub use self::text_block_type::TextBlockType;
pub use self::user_type::UserType;
