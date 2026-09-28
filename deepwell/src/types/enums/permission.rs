/*
 * types/enums/permission.rs
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

use sea_orm::DeriveValueType;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    DeriveValueType,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumString,
    Display,
    Serialize,
)]
#[sea_orm(value_type = "String")]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
#[serde(rename_all = "kebab-case")]
pub enum Resource {
    Page,
    Role,
    Site,
    PageLock,
}

#[derive(
    DeriveValueType,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumString,
    Display,
    Serialize,
)]
#[sea_orm(value_type = "String")]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    View,
    Edit,
    BypassLock,
    Create,
    Delete,
    Rename,
    Assign,
}
