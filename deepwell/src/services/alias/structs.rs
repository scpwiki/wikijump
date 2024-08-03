/*
 * services/alias/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

use crate::models::sea_orm_active_enums::AliasType;

#[derive(Deserialize, Debug)]
pub struct CreateAlias {
    pub slug: String,
    pub alias_type: AliasType,
    pub target_id: i64,
    pub created_by: i64,

    #[serde(default)]
    pub bypass_filter: bool,
}

#[derive(Serialize, Debug)]
pub struct CreateAliasOutput {
    pub alias_id: i64,
    pub slug: String,
}
