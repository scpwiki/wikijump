/*
 * services/relation/page_watch.rs
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

use super::RelationService;
use super::structs::{RelationDirection, RelationObject, RelationReference};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::relation::Model as RelationModel;
use crate::services::ServiceContext;
use crate::types::RelationType;
use paste::paste;

impl_relation!(PageWatch, Page, page_id, User, user_id, ());
